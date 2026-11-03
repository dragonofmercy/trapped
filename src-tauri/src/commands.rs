use crate::db::{Db, InboxRow, MailRow};
use crate::{parse, smtp};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;

const BLOCKED: [&str; 13] = ["exe", "bat", "cmd", "com", "ps1", "msi", "js", "vbs", "scr", "jar", "lnk", "reg", "hta"];

pub type DbState = Mutex<Db>;
pub type StatusState = Mutex<SmtpStatus>;
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
    pub raw: String,
    pub headers: Vec<(String, String)>,
    pub attachments: Vec<AttachmentInfo>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmtpStatus {
    pub listening: bool,
    pub address: String,
    pub error: Option<String>,
}

pub fn is_blocked_extension(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| BLOCKED.contains(&ext.to_ascii_lowercase().as_str()))
}

/// Binds the SMTP port; on success starts serving. Returns the resulting status.
pub async fn start_smtp(tx: UnboundedSender<smtp::Received>) -> SmtpStatus {
    match TcpListener::bind(smtp::ADDR).await {
        Ok(listener) => {
            tauri::async_runtime::spawn(smtp::serve(listener, tx));
            SmtpStatus {
                listening: true,
                address: smtp::ADDR.into(),
                error: None,
            }
        }
        Err(e) => SmtpStatus {
            listening: false,
            address: smtp::ADDR.into(),
            error: Some(e.to_string()),
        },
    }
}

fn e<T: ToString>(err: T) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_inboxes(db: State<DbState>) -> Result<Vec<InboxRow>, String> {
    db.lock().unwrap().list_inboxes().map_err(e)
}

#[tauri::command]
pub fn list_mails(db: State<DbState>, inbox_id: i64, query: String) -> Result<Vec<MailRow>, String> {
    db.lock().unwrap().list_mails(inbox_id, &query).map_err(e)
}

#[tauri::command]
pub fn get_mail(db: State<DbState>, id: i64) -> Result<MailDetail, String> {
    let m = db.lock().unwrap().get_mail(id).map_err(e)?;
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
        raw: String::from_utf8_lossy(&m.raw).into_owned(),
        headers: d.headers,
        attachments: d
            .attachments
            .iter()
            .map(|a| AttachmentInfo {
                index: a.index,
                name: a.name.clone(),
                mime: a.mime.clone(),
                size: a.data.len(),
                blocked: is_blocked_extension(&a.name),
            })
            .collect(),
    })
}

#[tauri::command]
pub fn delete_mails(db: State<DbState>, ids: Vec<i64>) -> Result<(), String> {
    db.lock().unwrap().delete_mails(&ids).map_err(e)
}

#[tauri::command]
pub fn empty_inbox(db: State<DbState>, inbox_id: i64) -> Result<(), String> {
    db.lock().unwrap().empty_inbox(inbox_id).map_err(e)
}

#[tauri::command]
pub fn rename_inbox(db: State<DbState>, inbox_id: i64, name: String) -> Result<(), String> {
    db.lock().unwrap().rename_inbox(inbox_id, &name)
}

#[tauri::command]
pub fn delete_inbox(db: State<DbState>, inbox_id: i64) -> Result<(), String> {
    db.lock().unwrap().delete_inbox(inbox_id)
}

#[tauri::command]
pub fn smtp_status(status: State<StatusState>) -> SmtpStatus {
    status.lock().unwrap().clone()
}

#[tauri::command]
pub async fn smtp_retry(app: AppHandle) -> Result<SmtpStatus, String> {
    let current = app.state::<StatusState>().lock().unwrap().clone();
    if current.listening {
        return Ok(current);
    }
    let status = start_smtp(app.state::<SmtpTx>().0.clone()).await;
    *app.state::<StatusState>().lock().unwrap() = status.clone();
    let _ = app.emit("smtp-status", status.clone());
    Ok(status)
}

fn attachment_data(app: &AppHandle, mail_id: i64, index: usize) -> Result<parse::Attachment, String> {
    let m = app.state::<DbState>().lock().unwrap().get_mail(mail_id).map_err(e)?;
    parse::detail(&m.raw).attachments.into_iter().find(|a| a.index == index).ok_or_else(|| "Attachment not found".to_string())
}

#[tauri::command]
pub fn open_attachment(app: AppHandle, mail_id: i64, index: usize) -> Result<(), String> {
    let a = attachment_data(&app, mail_id, index)?;
    if is_blocked_extension(&a.name) {
        return Err("This file type can only be saved".into());
    }
    let dir = app.path().app_cache_dir().map_err(e)?.join("attachments");
    std::fs::create_dir_all(&dir).map_err(e)?;
    let safe: String = a.name.chars().map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c }).collect();
    let path = dir.join(format!("{mail_id}-{index}-{safe}"));
    std::fs::write(&path, &a.data).map_err(e)?;
    tauri_plugin_opener::OpenerExt::opener(&app).open_path(path.to_string_lossy(), None::<&str>).map_err(e)
}

#[tauri::command]
pub fn save_attachment(app: AppHandle, mail_id: i64, index: usize, path: String) -> Result<(), String> {
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
        assert!(!is_blocked_extension("invoice.pdf"));
        assert!(!is_blocked_extension("noext"));
    }
}
