use base64::{Engine, engine::general_purpose::STANDARD};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

pub const MAX_SIZE: usize = 52_428_800;

pub struct Received {
    pub inbox: String,
    #[allow(dead_code)]
    pub mail_from: String,
    pub rcpt_to: Vec<String>,
    pub raw: Vec<u8>,
}

pub async fn serve(listener: TcpListener, tx: UnboundedSender<Received>) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle(stream, tx.clone()));
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
        }
    }
}

async fn read_line<R: AsyncBufReadExt + Unpin>(r: &mut R) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    if r.read_until(b'\n', &mut buf).await.ok()? == 0 {
        return None;
    }
    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    Some(buf)
}

/// Extracts the address from `FROM:<x> PARAMS` / `TO:<x>` (text after the keyword).
fn address(arg: &str) -> String {
    let rest = arg.split_once(':').map_or("", |(_, r)| r).trim();
    match (rest.find('<'), rest.find('>')) {
        (Some(a), Some(b)) if a < b => rest[a + 1..b].trim().to_string(),
        _ => rest.split_whitespace().next().unwrap_or("").to_string(),
    }
}

fn b64(line: &[u8]) -> Option<Vec<u8>> {
    // "=" is the SASL marker for an empty response
    if line == b"=" {
        return Some(Vec::new());
    }
    STANDARD.decode(line).ok()
}

pub async fn handle<S: AsyncRead + AsyncWrite + Unpin>(stream: S, tx: UnboundedSender<Received>) {
    let (rd, mut wr) = tokio::io::split(stream);
    let mut rd = BufReader::new(rd);
    let _ = session(&mut rd, &mut wr, &tx).await;
}

async fn session<R: AsyncBufReadExt + Unpin, W: AsyncWrite + Unpin>(rd: &mut R, wr: &mut W, tx: &UnboundedSender<Received>) -> Option<()> {
    macro_rules! say {
        ($s:expr) => {
            wr.write_all($s.as_bytes()).await.ok()?
        };
    }
    let mut inbox = String::new();
    let mut mail_from = String::new();
    let mut rcpt_to: Vec<String> = Vec::new();
    say!("220 Trapped ESMTP\r\n");
    loop {
        let line = String::from_utf8_lossy(&read_line(rd).await?).into_owned();
        let (verb, arg) = line.trim().split_once(' ').unwrap_or((line.trim(), ""));
        match verb.to_ascii_uppercase().as_str() {
            "EHLO" => say!("250-Trapped\r\n250-SIZE 52428800\r\n250-AUTH PLAIN LOGIN\r\n250 8BITMIME\r\n"),
            "HELO" => say!("250 Trapped\r\n"),
            "AUTH" => {
                let (mech, initial) = arg.split_once(' ').unwrap_or((arg, ""));
                let user = match mech.to_ascii_uppercase().as_str() {
                    "PLAIN" => {
                        let resp = if initial.is_empty() {
                            say!("334 \r\n");
                            read_line(rd).await?
                        } else {
                            initial.trim().as_bytes().to_vec()
                        };
                        // authzid \0 user \0 pass
                        b64(&resp).and_then(|d| d.split(|&c| c == 0).nth(1).map(|u| String::from_utf8_lossy(u).into_owned()))
                    }
                    "LOGIN" => {
                        let resp = if initial.is_empty() {
                            say!("334 VXNlcm5hbWU6\r\n");
                            read_line(rd).await?
                        } else {
                            initial.trim().as_bytes().to_vec()
                        };
                        let user = b64(&resp).map(|d| String::from_utf8_lossy(&d).into_owned());
                        if user.is_some() {
                            say!("334 UGFzc3dvcmQ6\r\n");
                            read_line(rd).await?; // password ignored, may be empty
                        }
                        user
                    }
                    _ => {
                        say!("504 5.5.4 Unrecognized mechanism\r\n");
                        continue;
                    }
                };
                match user {
                    Some(u) => {
                        inbox = u;
                        say!("235 2.7.0 Authentication successful\r\n");
                    }
                    None => say!("501 5.5.2 Cannot decode response\r\n"),
                }
            }
            "MAIL" => {
                let too_big = arg.split_whitespace().filter_map(|p| p.to_ascii_uppercase().strip_prefix("SIZE=").and_then(|n| n.parse::<usize>().ok())).any(|n| n > MAX_SIZE);
                if too_big {
                    say!("552 5.3.4 Message size exceeds limit\r\n");
                } else {
                    mail_from = address(arg);
                    rcpt_to.clear();
                    say!("250 2.1.0 OK\r\n");
                }
            }
            "RCPT" => {
                rcpt_to.push(address(arg));
                say!("250 2.1.5 OK\r\n");
            }
            "DATA" => {
                if rcpt_to.is_empty() {
                    say!("503 5.5.1 Need RCPT first\r\n");
                    continue;
                }
                say!("354 End data with <CR><LF>.<CR><LF>\r\n");
                let mut raw = Vec::new();
                let mut over = false;
                loop {
                    let l = read_line(rd).await?;
                    if l == b"." {
                        break;
                    }
                    if over {
                        continue;
                    }
                    raw.extend_from_slice(if l.starts_with(b"..") { &l[1..] } else { &l });
                    raw.extend_from_slice(b"\r\n");
                    over = raw.len() > MAX_SIZE;
                }
                if over {
                    say!("552 5.3.4 Message size exceeds limit\r\n");
                    mail_from.clear();
                    rcpt_to.clear();
                } else {
                    let _ = tx.send(Received {
                        inbox: inbox.clone(),
                        mail_from: std::mem::take(&mut mail_from),
                        rcpt_to: std::mem::take(&mut rcpt_to),
                        raw,
                    });
                    say!("250 2.0.0 OK\r\n");
                }
            }
            "RSET" => {
                mail_from.clear();
                rcpt_to.clear();
                say!("250 2.0.0 OK\r\n");
            }
            "NOOP" => say!("250 2.0.0 OK\r\n"),
            "QUIT" => {
                say!("221 2.0.0 Bye\r\n");
                return Some(());
            }
            _ => say!("502 5.5.2 Command not recognized\r\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{DuplexStream, ReadHalf, WriteHalf, duplex, split};
    use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

    struct Client {
        r: BufReader<ReadHalf<DuplexStream>>,
        w: WriteHalf<DuplexStream>,
        rx: UnboundedReceiver<Received>,
    }

    impl Client {
        async fn new() -> Client {
            let (a, b) = duplex(1 << 20);
            let (tx, rx) = unbounded_channel();
            tokio::spawn(handle(b, tx));
            let (r, w) = split(a);
            let mut c = Client { r: BufReader::new(r), w, rx };
            assert!(c.reply().await.starts_with("220 Trapped"));
            c
        }
        async fn raw(&mut self, s: &str) {
            self.w.write_all(s.as_bytes()).await.unwrap();
        }
        /// Reads a full (possibly multi-line) reply; empty string on EOF.
        async fn reply(&mut self) -> String {
            let mut out = String::new();
            loop {
                let mut l = String::new();
                if self.r.read_line(&mut l).await.unwrap() == 0 {
                    return out;
                }
                out.push_str(&l);
                if l.as_bytes().get(3) != Some(&b'-') {
                    return out;
                }
            }
        }
        async fn cmd(&mut self, line: &str) -> String {
            self.raw(&format!("{line}\r\n")).await;
            self.reply().await
        }
        async fn send_mail(&mut self, rcpt: &str, body_lines: &[&str]) -> String {
            assert!(self.cmd("MAIL FROM:<s@x>").await.starts_with("250"));
            assert!(self.cmd(&format!("RCPT TO:<{rcpt}>")).await.starts_with("250"));
            assert!(self.cmd("DATA").await.starts_with("354"));
            for l in body_lines {
                self.raw(&format!("{l}\r\n")).await;
            }
            self.cmd(".").await
        }
    }

    #[tokio::test]
    async fn plain_session_default_inbox() {
        let mut c = Client::new().await;
        let r = c.cmd("EHLO me").await;
        assert!(r.contains("250-SIZE 52428800") && r.contains("250-AUTH PLAIN LOGIN") && r.ends_with("250 8BITMIME\r\n"));
        assert!(c.send_mail("a@x", &["Subject: hi", "", "body"]).await.starts_with("250 2.0.0 OK"));
        let m = c.rx.try_recv().unwrap();
        assert_eq!(m.inbox, "");
        assert_eq!(m.mail_from, "s@x");
        assert_eq!(m.rcpt_to, vec!["a@x"]);
        assert_eq!(m.raw, b"Subject: hi\r\n\r\nbody\r\n");
        assert!(c.cmd("QUIT").await.starts_with("221"));
    }

    #[tokio::test]
    async fn auth_login_empty_password() {
        let mut c = Client::new().await;
        c.cmd("EHLO me").await;
        assert_eq!(c.cmd("AUTH LOGIN").await, "334 VXNlcm5hbWU6\r\n");
        assert_eq!(c.cmd(&STANDARD.encode("Inbox-Name")).await, "334 UGFzc3dvcmQ6\r\n");
        assert!(c.cmd("").await.starts_with("235 2.7.0"));
        c.send_mail("a@x", &["hi"]).await;
        assert_eq!(c.rx.try_recv().unwrap().inbox, "Inbox-Name");
    }

    #[tokio::test]
    async fn auth_login_initial_response() {
        let mut c = Client::new().await;
        assert_eq!(c.cmd(&format!("AUTH LOGIN {}", STANDARD.encode("u1"))).await, "334 UGFzc3dvcmQ6\r\n");
        assert!(c.cmd("=").await.starts_with("235"));
        c.send_mail("a@x", &["hi"]).await;
        assert_eq!(c.rx.try_recv().unwrap().inbox, "u1");
    }

    #[tokio::test]
    async fn auth_plain_initial_response() {
        let mut c = Client::new().await;
        c.cmd("EHLO me").await;
        assert!(c.cmd(&format!("AUTH PLAIN {}", STANDARD.encode("\0shop-api\0x"))).await.starts_with("235 2.7.0"));
        c.send_mail("a@x", &["hi"]).await;
        assert_eq!(c.rx.try_recv().unwrap().inbox, "shop-api");
    }

    #[tokio::test]
    async fn auth_plain_continuation_and_bad_base64() {
        let mut c = Client::new().await;
        assert!(c.cmd("AUTH PLAIN").await.starts_with("334"));
        assert!(c.cmd(&STANDARD.encode("\0box\0")).await.starts_with("235"));
        assert!(c.cmd("AUTH PLAIN !!!notbase64").await.starts_with("501"));
        c.send_mail("a@x", &["hi"]).await;
        assert_eq!(c.rx.try_recv().unwrap().inbox, "box");
    }

    #[tokio::test]
    async fn dot_unstuffing_and_bare_lf() {
        let mut c = Client::new().await;
        c.cmd("MAIL FROM:<s@x>").await;
        c.cmd("RCPT TO:<a@x>").await;
        c.cmd("DATA").await;
        c.raw("..leading\nsecond\n").await;
        assert!(c.cmd(".").await.starts_with("250"));
        let m = c.rx.try_recv().unwrap();
        assert_eq!(m.raw, b".leading\r\nsecond\r\n");
    }

    #[tokio::test]
    async fn two_mails_one_connection() {
        let mut c = Client::new().await;
        c.cmd(&format!("AUTH PLAIN {}", STANDARD.encode("\0keep\0"))).await;
        c.send_mail("a@x", &["one"]).await;
        c.cmd("RSET").await;
        c.send_mail("b@x", &["two"]).await;
        let (m1, m2) = (c.rx.try_recv().unwrap(), c.rx.try_recv().unwrap());
        assert_eq!((m1.inbox.as_str(), m2.inbox.as_str()), ("keep", "keep"));
        assert_eq!((m1.rcpt_to, m2.rcpt_to), (vec!["a@x".to_string()], vec!["b@x".to_string()]));
    }

    #[tokio::test]
    async fn oversize_rejected() {
        let mut c = Client::new().await;
        assert!(c.cmd("MAIL FROM:<a> SIZE=99999999").await.starts_with("552"));
        assert!(c.cmd("mail from:<a> size=10").await.starts_with("250"));
    }

    #[tokio::test]
    async fn data_without_rcpt() {
        let mut c = Client::new().await;
        c.cmd("MAIL FROM:<a>").await;
        assert!(c.cmd("DATA").await.starts_with("503"));
        assert!(c.cmd("BOGUS").await.starts_with("502"));
    }
}
