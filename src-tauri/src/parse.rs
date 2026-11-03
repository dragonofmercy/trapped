use base64::Engine;
use mail_parser::{Addr, Address, Message, MessageParser, MimeHeaders, PartType};

pub struct Summary {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub has_attachments: bool,
    pub search_text: String,
}

pub struct Attachment {
    pub index: usize,
    pub name: String,
    pub mime: String,
    pub content_id: Option<String>,
    pub data: Vec<u8>,
}

pub struct Detail {
    pub date: Option<String>,
    pub html: Option<String>,
    pub text: Option<String>,
    pub headers: Vec<(String, String)>,
    pub attachments: Vec<Attachment>,
}

fn emails(addr: Option<&Address>) -> Vec<String> {
    addr.map(|a| a.iter().filter_map(|x: &Addr| x.address.as_deref()).map(str::to_string).collect()).unwrap_or_default()
}

/// First text/plain body part only (no html-to-text fallback).
fn text_body(msg: &Message) -> Option<String> {
    match &msg.text_part(0)?.body {
        PartType::Text(t) => Some(t.to_string()),
        _ => None,
    }
}

/// First text/html body part only (no text-to-html fallback).
fn html_body(msg: &Message) -> Option<String> {
    match &msg.html_part(0)?.body {
        PartType::Html(h) => Some(h.to_string()),
        _ => None,
    }
}

/// A part counts as an attachment when it has a filename or an explicit attachment disposition.
fn is_listed(part: &mail_parser::MessagePart) -> bool {
    if part.content_disposition().is_some_and(|d| d.is_attachment()) {
        return true;
    }
    // An inline image referenced by cid is not a user-facing attachment even when it carries a filename.
    part.attachment_name().is_some() && part.content_id().is_none()
}

pub fn summarize(raw: &[u8], rcpt_to: &[String]) -> Summary {
    let msg = MessageParser::default().parse(raw);
    let msg = msg.as_ref();
    let subject = msg.and_then(|m| m.subject()).filter(|s| !s.trim().is_empty()).map(str::to_string).unwrap_or_else(|| "(no subject)".to_string());
    let from = msg.map(|m| emails(m.from())).and_then(|v| v.into_iter().next()).unwrap_or_default();
    let to = msg.map(|m| emails(m.to())).unwrap_or_default();
    let cc = msg.map(|m| emails(m.cc())).unwrap_or_default();
    let bcc: Vec<String> = rcpt_to.iter().filter(|r| !to.iter().chain(cc.iter()).any(|h| h.eq_ignore_ascii_case(r))).cloned().collect();
    let has_attachments = msg.is_some_and(|m| m.attachments().any(is_listed));
    let text = msg.and_then(text_body).unwrap_or_default();
    let html = msg.and_then(html_body).map(|h| strip_html(&h)).unwrap_or_default();
    let mut parts = vec![subject.clone(), from.clone()];
    parts.extend(to.iter().chain(cc.iter()).chain(bcc.iter()).cloned());
    parts.push(text);
    parts.push(html);
    let search_text = parts.join(" ").to_lowercase();
    Summary {
        subject,
        from,
        to,
        cc,
        bcc,
        has_attachments,
        search_text,
    }
}

pub fn detail(raw: &[u8]) -> Detail {
    let Some(msg) = MessageParser::default().parse(raw) else {
        return Detail {
            date: None,
            html: None,
            text: None,
            headers: vec![],
            attachments: vec![],
        };
    };
    // Inline cid parts are needed for substitution but are not listed to the user.
    let mut all = Vec::new();
    let mut flags = Vec::new();
    for part in msg.attachments() {
        let mime = part.content_type().map(|c| format!("{}/{}", c.ctype(), c.subtype().unwrap_or("octet-stream"))).unwrap_or_else(|| "application/octet-stream".to_string());
        flags.push(is_listed(part));
        all.push(Attachment {
            index: 0,
            name: part.attachment_name().unwrap_or("attachment").to_string(),
            mime,
            content_id: part.content_id().map(|c| c.trim_matches(|ch| ch == '<' || ch == '>').to_string()),
            data: part.contents().to_vec(),
        });
    }
    let html = html_body(&msg).map(|h| inline_cids(&h, &all));
    Detail {
        date: msg.date().map(|d| d.to_rfc3339()),
        html,
        text: text_body(&msg),
        headers: msg.headers_raw().map(|(k, v)| (k.to_string(), v.trim().to_string())).collect(),
        attachments: all.into_iter().zip(flags).filter(|(_, l)| *l).enumerate().map(|(i, (a, _))| Attachment { index: i, ..a }).collect(),
    }
}

pub fn strip_html(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        if html.as_bytes()[i] == b'<' {
            let close = html[i..].find('>').map_or(html.len(), |p| i + p + 1);
            let name: String = lower[i + 1..close].chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            i = close;
            if name == "style" || name == "script" {
                let end = format!("</{name}");
                i = lower[i..].find(&end).map_or(html.len(), |p| i + p);
            }
            out.push(' ');
        } else {
            let next = html[i..].find('<').map_or(html.len(), |p| i + p);
            out.push_str(&html[i..next]);
            i = next;
        }
    }
    let decoded = out.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Replaces `needle` only when followed by a delimiter, so "cid:logo1" never matches inside "cid:logo10".
fn replace_cid(html: &str, needle: &str, uri: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(p) = rest.find(needle) {
        let after = &rest[p + needle.len()..];
        out.push_str(&rest[..p]);
        if after.chars().next().is_none_or(|c| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | '>')) {
            out.push_str(uri);
        } else {
            out.push_str(needle);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

pub fn inline_cids(html: &str, attachments: &[Attachment]) -> String {
    let mut out = html.to_string();
    for a in attachments {
        if let Some(id) = a.content_id.as_deref() {
            let id = id.trim_matches(|c| c == '<' || c == '>');
            let uri = format!("data:{};base64,{}", a.mime, base64::engine::general_purpose::STANDARD.encode(&a.data));
            out = replace_cid(&out, &format!("cid:{id}"), &uri);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MULTIPART: &[u8] = b"From: App <noreply@app.test>\r\nTo: a@x.test\r\nCc: c@x.test\r\nSubject: Confirm your email\r\nMIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nplease verify my email\r\n--b\r\nContent-Type: text/html\r\n\r\n<p><a href=\"http://x.test\">verify my email</a></p>\r\n--b--\r\n";
    const HTML_ONLY: &[u8] = b"From: a@x.test\r\nSubject: h\r\nContent-Type: text/html\r\n\r\n<p>Hi</p>\r\n";
    const TEXT_ONLY: &[u8] = b"From: a@x.test\r\nSubject: t\r\nContent-Type: text/plain\r\n\r\nHello\r\n";
    const WITH_PDF: &[u8] = b"From: a@x.test\r\nSubject: p\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nSee attached\r\n--b\r\nContent-Type: application/pdf; name=\"invoice.pdf\"\r\nContent-Disposition: attachment; filename=\"invoice.pdf\"\r\nContent-Transfer-Encoding: base64\r\n\r\nJVBERi0xLjQK\r\n--b--\r\n";
    const WITH_CID_IMAGE: &[u8] = b"From: a@x.test\r\nSubject: i\r\nMIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"b\"\r\n\r\n--b\r\nContent-Type: text/html\r\n\r\n<img src=\"cid:logo1\">\r\n--b\r\nContent-Type: image/png\r\nContent-ID: <logo1>\r\nContent-Disposition: inline\r\nContent-Transfer-Encoding: base64\r\n\r\niVBORw0KGgo=\r\n--b--\r\n";

    fn rcpt(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn summary_basic() {
        let s = summarize(MULTIPART, &rcpt(&["a@x.test", "c@x.test", "hidden@x.test"]));
        assert_eq!(s.subject, "Confirm your email");
        assert_eq!(s.from, "noreply@app.test");
        assert_eq!(s.to, vec!["a@x.test"]);
        assert_eq!(s.cc, vec!["c@x.test"]);
        assert_eq!(s.bcc, vec!["hidden@x.test"]);
        assert!(!s.has_attachments);
        assert!(s.search_text.contains("verify my email"));
        assert!(!s.search_text.contains("<a"));
    }

    #[test]
    fn summary_bcc_case_insensitive() {
        let s = summarize(MULTIPART, &rcpt(&["A@X.test"]));
        assert!(s.bcc.is_empty());
    }

    #[test]
    fn broken_mime_falls_back() {
        let s = summarize(b"\xff\xfe garbage without headers", &[]);
        assert_eq!(s.subject, "(no subject)");
        assert_eq!(s.from, "");
    }

    #[test]
    fn detail_html_only() {
        let d = detail(HTML_ONLY);
        assert!(d.html.is_some());
        assert!(d.text.is_none());
    }

    #[test]
    fn detail_text_only() {
        let d = detail(TEXT_ONLY);
        assert!(d.html.is_none());
        assert_eq!(d.text.as_deref().map(str::trim), Some("Hello"));
    }

    #[test]
    fn detail_no_body() {
        let d = detail(b"Subject: x\r\n\r\n");
        assert!(d.attachments.is_empty());
        assert!(d.html.is_none());
    }

    #[test]
    fn attachment_listed() {
        let d = detail(WITH_PDF);
        assert_eq!(d.attachments.len(), 1);
        assert_eq!(d.attachments[0].name, "invoice.pdf");
        assert_eq!(d.attachments[0].mime, "application/pdf");
        assert!(summarize(WITH_PDF, &[]).has_attachments);
    }

    #[test]
    fn cid_inlined() {
        let d = detail(WITH_CID_IMAGE);
        let h = d.html.unwrap();
        assert!(!h.contains("cid:"));
        assert!(h.contains("data:image/png;base64,"));
        assert!(d.attachments.is_empty());
        assert!(!summarize(WITH_CID_IMAGE, &[]).has_attachments);
    }

    #[test]
    fn cid_prefix_and_brackets() {
        let raw: &[u8] = b"From: a@x.test
Subject: i
MIME-Version: 1.0
Content-Type: multipart/related; boundary=\"b\"

--b
Content-Type: text/html

<img src=\"cid:logo1\"><img src='cid:logo10'>
--b
Content-Type: image/png
Content-ID: <logo1>
Content-Transfer-Encoding: base64

AAAA
--b
Content-Type: image/gif
Content-ID: logo10
Content-Transfer-Encoding: base64

BBBB
--b--
";
        let h = detail(raw).html.unwrap();
        assert!(!h.contains("cid:"), "{h}");
        assert!(h.contains("\"data:image/png;base64,AAAA\""), "{h}");
        assert!(h.contains("'data:image/gif;base64,BBBB'"), "{h}");
    }

    #[test]
    fn inline_image_with_filename_not_listed() {
        let raw: &[u8] = b"From: a@x.test
Subject: i
MIME-Version: 1.0
Content-Type: multipart/related; boundary=\"b\"

--b
Content-Type: text/html

<img src=\"cid:logo1\">
--b
Content-Type: image/png; name=\"logo.png\"
Content-ID: <logo1>
Content-Disposition: inline; filename=\"logo.png\"
Content-Transfer-Encoding: base64

AAAA
--b--
";
        let d = detail(raw);
        assert!(d.attachments.is_empty());
        assert!(!summarize(raw, &[]).has_attachments);
        assert!(d.html.unwrap().contains("data:image/png;base64,"));
    }

    #[test]
    fn empty_subject_falls_back() {
        assert_eq!(
            summarize(
                b"Subject:

x",
                &[]
            )
            .subject,
            "(no subject)"
        );
    }

    #[test]
    fn strip_html_basic() {
        assert_eq!(strip_html("<style>p{}</style><p>A &amp; B</p>\n<b>c</b>"), "A & B c");
    }
}
