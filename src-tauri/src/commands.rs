use crate::db::{Db, InboxRow, MailRow, SmtpHost};
use crate::{parse, smtp};
use serde::Serialize;
use std::num::NonZeroU16;
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

const BLOCKED: [&str; 21] = ["exe", "bat", "cmd", "com", "ps1", "msi", "js", "vbs", "scr", "jar", "lnk", "reg", "hta", "msc", "cpl", "wsf", "psm1", "url", "chm", "appref-ms", "pif"];

pub type DbState = Mutex<Db>;
pub type SmtpState = tokio::sync::Mutex<Smtp>;

/// The running SMTP listener (if any) and its status as shown in the UI
pub struct Smtp {
    pub status: SmtpStatus,
    host: SmtpHost,
    port: u16,
    task: Option<tauri::async_runtime::JoinHandle<()>>,
}

/// A poisoned lock only means a panic happened mid-query; the connection itself is still usable
pub fn lock(db: &DbState) -> MutexGuard<'_, Db> {
    db.lock().unwrap_or_else(|p| p.into_inner())
}

fn address(host: SmtpHost, port: u16) -> String {
    format!("{}:{port}", host.ip())
}
pub struct SmtpTx(pub UnboundedSender<smtp::Received>);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub index: usize,
    pub name: String,
    pub mime: String,
    pub size: usize,
    pub blocked: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailDetail {
    pub id: i64,
    pub inbox_id: i64,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub date: Option<String>,
    pub received_at: i64,
    pub html: Option<String>,
    pub text: Option<String>,
    pub headers: Vec<(String, String)>,
    pub attachments: Vec<AttachmentInfo>,
    pub was_unread: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmtpStatus {
    pub listening: bool,
    pub address: String,
    pub error: Option<String>,
}

pub fn is_blocked_extension(name: &str) -> bool {
    // Windows drops trailing dots and spaces when creating files, so "x.exe." still ends up as x.exe
    name.trim_end_matches(['.', ' ']).rsplit_once('.').is_some_and(|(_, ext)| BLOCKED.contains(&ext.to_ascii_lowercase().as_str()))
}

fn bind_error(host: SmtpHost, port: u16, err: std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::AddrInUse {
        format!("Port {port} is already in use (is another SMTP catcher like smtp4dev running?)")
    } else {
        format!("Cannot listen on {}: {err}", address(host, port))
    }
}

/// Binds host:port. On 127.0.0.1, Windows lets the socket coexist with another program listening on all interfaces
/// (Apache on 0.0.0.0:80), and the more specific bind would silently steal its local traffic. So the wildcard addresses
/// are probed first, with a bind and no listen (no firewall prompt). On 0.0.0.0 the bind itself catches the IPv4 conflicts.
async fn bind(host: SmtpHost, port: u16) -> std::io::Result<TcpListener> {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    if host == SmtpHost::Local {
        tokio::net::TcpSocket::new_v4()?.bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?;
    }
    // A missing IPv6 stack is not a conflict, only an IPv6 listener on the port is
    if let Ok(v6) = tokio::net::TcpSocket::new_v6()
        && let Err(err) = v6.bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)))
        && err.kind() == std::io::ErrorKind::AddrInUse
    {
        return Err(err);
    }
    TcpListener::bind((host.ip(), port)).await
}

impl Smtp {
    /// Serves on host:port, replacing the previous listener. When the bind fails the previous listener (if any) keeps running.
    async fn listen(&mut self, tx: UnboundedSender<smtp::Received>, host: SmtpHost, port: u16) -> Result<(), String> {
        // Same port on another address: our own listener would make the new bind fail, so it is stopped first and restored on failure
        let same_port = self.task.is_some() && self.port == port;
        if same_port && let Some(task) = self.task.take() {
            task.abort();
            let _ = task.await;
        }
        let listener = match bind(host, port).await {
            Ok(listener) => listener,
            Err(err) => {
                if same_port {
                    match bind(self.host, self.port).await {
                        Ok(old) => self.task = Some(tauri::async_runtime::spawn(smtp::serve(old, tx))),
                        Err(old) => {
                            self.status = SmtpStatus {
                                listening: false,
                                address: address(self.host, self.port),
                                error: Some(bind_error(self.host, self.port, old)),
                            }
                        }
                    }
                }
                return Err(bind_error(host, port, err));
            }
        };
        if let Some(task) = self.task.take() {
            task.abort();
        }
        self.task = Some(tauri::async_runtime::spawn(smtp::serve(listener, tx)));
        self.host = host;
        self.port = port;
        self.status = SmtpStatus {
            listening: true,
            address: address(host, port),
            error: None,
        };
        Ok(())
    }

    /// Start or retry: a failed bind is reported through the status (the UI offers Retry)
    async fn listen_or_report(&mut self, tx: UnboundedSender<smtp::Received>, host: SmtpHost, port: u16) {
        if let Err(err) = self.listen(tx, host, port).await {
            self.host = host;
            self.port = port;
            self.status = SmtpStatus {
                listening: false,
                address: address(host, port),
                error: Some(err),
            };
        }
    }

    pub async fn start(tx: UnboundedSender<smtp::Received>, host: SmtpHost, port: u16) -> Smtp {
        let mut smtp = Smtp {
            status: SmtpStatus {
                listening: false,
                address: address(host, port),
                error: None,
            },
            host,
            port,
            task: None,
        };
        smtp.listen_or_report(tx, host, port).await;
        smtp
    }
}

#[tauri::command]
pub async fn list_inboxes(db: State<'_, DbState>) -> Result<Vec<InboxRow>, String> {
    lock(&db).list_inboxes().map_err(e)
}

#[tauri::command]
pub async fn list_mails(db: State<'_, DbState>, inbox_id: i64, query: String) -> Result<Vec<MailRow>, String> {
    lock(&db).list_mails(inbox_id, &query).map_err(e)
}

#[tauri::command]
pub async fn get_mail(app: AppHandle, id: i64) -> Result<MailDetail, String> {
    let m = lock(&app.state::<DbState>()).get_mail(id).map_err(e)?;
    let d = parse::detail(&m.raw);
    Ok(MailDetail {
        id: m.id,
        inbox_id: m.inbox_id,
        subject: m.subject,
        from: m.from,
        to: m.to,
        cc: m.cc,
        bcc: m.bcc,
        date: d.date,
        received_at: m.received_at,
        html: d.html,
        text: d.text,
        headers: d.headers,
        was_unread: m.was_unread,
        attachments: d
            .attachments
            .into_iter()
            .map(|a| AttachmentInfo {
                blocked: is_blocked_extension(&a.name),
                index: a.index,
                name: a.name,
                mime: a.mime,
                size: a.size,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn get_raw(db: State<'_, DbState>, id: i64) -> Result<String, String> {
    let raw = lock(&db).get_raw(id).map_err(e)?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

#[tauri::command]
pub async fn delete_mails(db: State<'_, DbState>, ids: Vec<i64>) -> Result<(), String> {
    lock(&db).delete_mails(&ids).map_err(e)
}

#[tauri::command]
pub async fn rename_inbox(db: State<'_, DbState>, inbox_id: i64, name: String) -> Result<(), String> {
    lock(&db).rename_inbox(inbox_id, &name)
}

#[tauri::command]
pub async fn create_inbox(db: State<'_, DbState>, name: String) -> Result<i64, String> {
    lock(&db).create_inbox(&name)
}

#[tauri::command]
pub async fn delete_inbox(db: State<'_, DbState>, inbox_id: i64) -> Result<(), String> {
    lock(&db).delete_inbox(inbox_id)
}

#[tauri::command]
pub async fn smtp_status(smtp: State<'_, SmtpState>) -> Result<SmtpStatus, String> {
    Ok(smtp.lock().await.status.clone())
}

#[tauri::command]
pub async fn smtp_retry(app: AppHandle) -> Result<SmtpStatus, String> {
    let state = app.state::<SmtpState>();
    let mut smtp = state.lock().await;
    if !smtp.status.listening {
        let (host, port) = (smtp.host, smtp.port);
        smtp.listen_or_report(app.state::<SmtpTx>().0.clone(), host, port).await;
        let _ = app.emit("smtp-status", smtp.status.clone());
    }
    Ok(smtp.status.clone())
}

fn e<T: ToString>(err: T) -> String {
    err.to_string()
}

#[derive(Serialize)]
pub struct SmtpConfig {
    host: SmtpHost,
    port: u16,
}

#[tauri::command]
pub async fn get_smtp_config(smtp: State<'_, SmtpState>) -> Result<SmtpConfig, String> {
    let smtp = smtp.lock().await;
    Ok(SmtpConfig { host: smtp.host, port: smtp.port })
}

/// Tells the Options modal whether a port can be used: "current" (Trapped already listens on it), "free" or "busy".
/// Other bind failures (e.g. a port reserved by Windows) come back as an error message.
#[tauri::command]
pub async fn check_smtp_port(smtp: State<'_, SmtpState>, host: SmtpHost, port: NonZeroU16) -> Result<&'static str, String> {
    let port = port.get();
    let current = smtp.lock().await;
    if current.status.listening && current.port == port {
        return Ok("current");
    }
    drop(current);
    match bind(host, port).await {
        Ok(_) => Ok("free"),
        Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => Ok("busy"),
        Err(err) => Err(bind_error(host, port, err)),
    }
}

/// Moves the listener to the new address, then saves it. A failed bind leaves the saved address unchanged, and the listener too
/// unless it had to be stopped (same port) and could not be restored.
#[tauri::command]
pub async fn set_smtp_config(app: AppHandle, host: SmtpHost, port: NonZeroU16) -> Result<SmtpStatus, String> {
    let port = port.get();
    let state = app.state::<SmtpState>();
    let mut smtp = state.lock().await;
    if !(smtp.status.listening && smtp.host == host && smtp.port == port) {
        let result = smtp.listen(app.state::<SmtpTx>().0.clone(), host, port).await;
        let _ = app.emit("smtp-status", smtp.status.clone());
        result?;
        let db = app.state::<DbState>();
        let db = lock(&db);
        db.set_smtp_host(host).map_err(e)?;
        db.set_smtp_port(port).map_err(e)?;
    }
    Ok(smtp.status.clone())
}

#[tauri::command]
pub async fn get_check_updates(db: State<'_, DbState>) -> Result<bool, String> {
    lock(&db).check_updates().map_err(e)
}

#[tauri::command]
pub async fn set_check_updates(db: State<'_, DbState>, enabled: bool) -> Result<(), String> {
    lock(&db).set_check_updates(enabled).map_err(e)
}

fn attachment_data(app: &AppHandle, mail_id: i64, index: usize) -> Result<parse::Attachment, String> {
    let m = lock(&app.state::<DbState>()).get_raw(mail_id).map_err(e)?;
    parse::attachment(&m, index).ok_or_else(|| "Attachment not found".to_string())
}

#[tauri::command]
pub async fn open_attachment(app: AppHandle, mail_id: i64, index: usize) -> Result<(), String> {
    let a = attachment_data(&app, mail_id, index)?;
    let dir = app.path().app_cache_dir().map_err(e)?.join("attachments");
    std::fs::create_dir_all(&dir).map_err(e)?;
    let safe: String = a.name.chars().map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c }).collect();
    let safe = safe.trim_end_matches(['.', ' ']);
    let safe = if safe.is_empty() { "attachment" } else { safe };
    // check the name that is actually written, not only the declared one
    if is_blocked_extension(safe) || is_blocked_extension(&a.name) {
        return Err("This file type can only be saved".into());
    }
    let path = dir.join(format!("{mail_id}-{index}-{safe}"));
    std::fs::write(&path, &a.data).map_err(e)?;
    tauri_plugin_opener::OpenerExt::opener(&app).open_path(path.to_string_lossy(), None::<&str>).map_err(e)
}

#[tauri::command]
pub async fn save_attachment(app: AppHandle, mail_id: i64, index: usize, path: String) -> Result<(), String> {
    let a = attachment_data(&app, mail_id, index)?;
    std::fs::write(path, a.data).map_err(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_extensions() {
        assert!(is_blocked_extension("setup.EXE"));
        assert!(is_blocked_extension("a.tar.ps1"));
        assert!(is_blocked_extension("x.exe."));
        assert!(is_blocked_extension("x.EXE "));
        assert!(is_blocked_extension("x.exe. ."));
        assert!(is_blocked_extension("a.MSC"));
        assert!(is_blocked_extension("b.appref-ms"));
        assert!(!is_blocked_extension("invoice.pdf"));
        assert!(!is_blocked_extension("noext"));
    }
}
