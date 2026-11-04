use crate::parse::Summary;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::path::Path;

/// Created on first launch, and used for mail sent without a username. A normal inbox otherwise.
pub const DEFAULT_INBOX: &str = "Inbox-Name";
pub const DEFAULT_SMTP_PORT: u16 = 2525;

/// Address the SMTP listener binds: this machine only, or every network interface (mail from other machines, e.g. a shared Docker host)
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SmtpHost {
    #[default]
    #[serde(rename = "127.0.0.1")]
    Local,
    #[serde(rename = "0.0.0.0")]
    All,
}

impl SmtpHost {
    pub fn ip(self) -> Ipv4Addr {
        match self {
            SmtpHost::Local => Ipv4Addr::LOCALHOST,
            SmtpHost::All => Ipv4Addr::UNSPECIFIED,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InboxRow {
    pub id: i64,
    pub name: String,
    pub count: i64,
    pub unread: i64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MailRow {
    pub id: i64,
    pub subject: String,
    pub from: String,
    pub received_at: i64,
    pub has_attachments: bool,
    pub is_read: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoredMail {
    pub id: i64,
    pub inbox_id: i64,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub received_at: i64,
    pub raw: Vec<u8>,
    /// This read marked the mail as read (the inbox unread count changed)
    pub was_unread: bool,
}

pub struct Db {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS inboxes (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE COLLATE NOCASE);
CREATE TABLE IF NOT EXISTS mails (
    id INTEGER PRIMARY KEY,
    inbox_id INTEGER NOT NULL REFERENCES inboxes(id) ON DELETE CASCADE,
    subject TEXT NOT NULL, from_addr TEXT NOT NULL, to_json TEXT NOT NULL, cc_json TEXT NOT NULL, bcc_json TEXT NOT NULL,
    received_at INTEGER NOT NULL, has_attachments INTEGER NOT NULL, is_read INTEGER NOT NULL DEFAULT 0, search_text TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS raws (mail_id INTEGER PRIMARY KEY REFERENCES mails(id) ON DELETE CASCADE, raw BLOB NOT NULL);
CREATE INDEX IF NOT EXISTS idx_mails_inbox ON mails(inbox_id, received_at);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
";

fn json_list(s: String) -> Vec<String> {
    serde_json::from_str(&s).unwrap_or_default()
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Db> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Db> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Db> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Returns the resulting mode ("memory" for in-memory DBs), so read it instead of using pragma_update.
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
        conn.execute_batch(SCHEMA)?;
        // Seed the starter inbox on a fresh database only, so deleting every inbox sticks across restarts
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 0 {
            conn.execute("INSERT INTO inboxes (name) SELECT ?1 WHERE NOT EXISTS (SELECT 1 FROM inboxes)", [DEFAULT_INBOX])?;
            conn.pragma_update(None, "user_version", 1)?;
        }
        Ok(Db { conn })
    }

    fn setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0)).optional()
    }

    fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute("INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [key, value])?;
        Ok(())
    }

    pub fn smtp_port(&self) -> rusqlite::Result<u16> {
        Ok(self.setting("smtp_port")?.and_then(|v| v.parse().ok()).filter(|&p| p != 0).unwrap_or(DEFAULT_SMTP_PORT))
    }

    pub fn set_smtp_port(&self, port: u16) -> rusqlite::Result<()> {
        self.set_setting("smtp_port", &port.to_string())
    }

    pub fn smtp_host(&self) -> rusqlite::Result<SmtpHost> {
        Ok(if self.setting("smtp_host")?.as_deref() == Some("0.0.0.0") { SmtpHost::All } else { SmtpHost::Local })
    }

    pub fn set_smtp_host(&self, host: SmtpHost) -> rusqlite::Result<()> {
        self.set_setting("smtp_host", &host.ip().to_string())
    }

    pub fn inbox_id(&self, name: &str) -> rusqlite::Result<i64> {
        let name = match name.trim() {
            "" => DEFAULT_INBOX,
            n => n,
        };
        if let Some(id) = self.conn.query_row("SELECT id FROM inboxes WHERE name = ?1", [name], |r| r.get(0)).optional()? {
            return Ok(id);
        }
        self.conn.execute("INSERT INTO inboxes (name) VALUES (?1)", [name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_mail(&self, inbox_id: i64, summary: &Summary, raw: &[u8], received_at: i64) -> rusqlite::Result<i64> {
        let tx = self.conn.unchecked_transaction()?;
        let list = |v: &Vec<String>| serde_json::to_string(v).unwrap_or_else(|_| "[]".into());
        tx.execute(
            "INSERT INTO mails (inbox_id, subject, from_addr, to_json, cc_json, bcc_json, received_at, has_attachments, search_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![inbox_id, summary.subject, summary.from, list(&summary.to), list(&summary.cc), list(&summary.bcc), received_at, summary.has_attachments, summary.search_text],
        )?;
        let id = tx.last_insert_rowid();
        tx.execute("INSERT INTO raws (mail_id, raw) VALUES (?1, ?2)", params![id, raw])?;
        tx.commit()?;
        Ok(id)
    }

    pub fn list_inboxes(&self) -> rusqlite::Result<Vec<InboxRow>> {
        let mut st = self
            .conn
            .prepare("SELECT i.id, i.name, COUNT(m.id), COALESCE(SUM(1 - m.is_read), 0) FROM inboxes i LEFT JOIN mails m ON m.inbox_id = i.id GROUP BY i.id ORDER BY i.name ASC")?;
        st.query_map([], |r| {
            Ok(InboxRow {
                id: r.get(0)?,
                name: r.get(1)?,
                count: r.get(2)?,
                unread: r.get(3)?,
            })
        })?
        .collect()
    }

    pub fn list_mails(&self, inbox_id: i64, query: &str) -> rusqlite::Result<Vec<MailRow>> {
        let q = query.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let mut st = self
            .conn
            .prepare("SELECT id, subject, from_addr, received_at, has_attachments, is_read FROM mails WHERE inbox_id = ?1 AND (?2 = '' OR search_text LIKE '%' || ?2 || '%' ESCAPE '\\') ORDER BY received_at DESC, id DESC")?;
        st.query_map(params![inbox_id, q], |r| {
            Ok(MailRow {
                id: r.get(0)?,
                subject: r.get(1)?,
                from: r.get(2)?,
                received_at: r.get(3)?,
                has_attachments: r.get(4)?,
                is_read: r.get(5)?,
            })
        })?
        .collect()
    }

    /// Raw message only; unlike `get_mail` it does not mark the mail as read.
    pub fn get_raw(&self, id: i64) -> rusqlite::Result<Vec<u8>> {
        self.conn.query_row("SELECT raw FROM raws WHERE mail_id = ?1", [id], |r| r.get(0))
    }

    pub fn get_mail(&self, id: i64) -> rusqlite::Result<StoredMail> {
        let was_unread = self.conn.execute("UPDATE mails SET is_read = 1 WHERE id = ?1 AND is_read = 0", [id])? > 0;
        self.conn.query_row(
            "SELECT m.id, m.inbox_id, m.subject, m.from_addr, m.to_json, m.cc_json, m.bcc_json, m.received_at, r.raw FROM mails m JOIN raws r ON r.mail_id = m.id WHERE m.id = ?1",
            [id],
            |r| {
                Ok(StoredMail {
                    id: r.get(0)?,
                    inbox_id: r.get(1)?,
                    subject: r.get(2)?,
                    from: r.get(3)?,
                    to: json_list(r.get(4)?),
                    cc: json_list(r.get(5)?),
                    bcc: json_list(r.get(6)?),
                    received_at: r.get(7)?,
                    raw: r.get(8)?,
                    was_unread,
                })
            },
        )
    }

    pub fn delete_mails(&self, ids: &[i64]) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for id in ids {
            tx.execute("DELETE FROM mails WHERE id = ?1", [id])?;
        }
        tx.commit()
    }

    pub fn rename_inbox(&self, inbox_id: i64, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Inbox name cannot be empty".into());
        }
        let taken: bool = self.conn.query_row("SELECT EXISTS(SELECT 1 FROM inboxes WHERE name = ?1 AND id != ?2)", params![name, inbox_id], |r| r.get(0)).map_err(|e| e.to_string())?;
        if taken {
            return Err("An inbox with this name already exists".into());
        }
        self.conn.execute("UPDATE inboxes SET name = ?1 WHERE id = ?2", params![name, inbox_id]).map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn create_inbox(&self, name: &str) -> Result<i64, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Inbox name cannot be empty".into());
        }
        let taken: bool = self.conn.query_row("SELECT EXISTS(SELECT 1 FROM inboxes WHERE name = ?1)", [name], |r| r.get(0)).map_err(|e| e.to_string())?;
        if taken {
            return Err("An inbox with this name already exists".into());
        }
        self.inbox_id(name).map_err(|e| e.to_string())
    }

    pub fn delete_inbox(&self, inbox_id: i64) -> Result<(), String> {
        self.conn.execute("DELETE FROM inboxes WHERE id = ?1", [inbox_id]).map(|_| ()).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Summary;

    fn sum(subject: &str, search: &str) -> Summary {
        Summary {
            subject: subject.into(),
            from: "a@x.test".into(),
            to: vec!["b@x.test".into()],
            cc: vec![],
            bcc: vec![],
            has_attachments: false,
            search_text: search.into(),
        }
    }

    #[test]
    fn smtp_port_setting() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.smtp_port().unwrap(), DEFAULT_SMTP_PORT);
        db.set_smtp_port(2526).unwrap();
        db.set_smtp_port(1025).unwrap();
        assert_eq!(db.smtp_port().unwrap(), 1025);
    }

    #[test]
    fn smtp_host_setting() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.smtp_host().unwrap(), SmtpHost::Local);
        db.set_smtp_host(SmtpHost::All).unwrap();
        assert_eq!(db.smtp_host().unwrap(), SmtpHost::All);
        db.set_smtp_host(SmtpHost::Local).unwrap();
        assert_eq!(db.smtp_host().unwrap(), SmtpHost::Local);
    }

    #[test]
    fn starter_inbox_seeded_once() {
        let dir = std::env::temp_dir().join(format!("trapped-seed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.db");
        let _ = std::fs::remove_file(&path);
        let db = Db::open(&path).unwrap();
        let l = db.list_inboxes().unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].name, "Inbox-Name");
        // Deleting every inbox must stick across a reopen
        db.delete_inbox(l[0].id).unwrap();
        drop(db);
        assert!(Db::open(&path).unwrap().list_inboxes().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inbox_name_case_insensitive() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.inbox_id("Shop").unwrap(), db.inbox_id(" shop ").unwrap());
        let l = db.list_inboxes().unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[1].name, "Shop");
    }

    #[test]
    fn empty_name_is_default() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.inbox_id("").unwrap(), db.inbox_id("Inbox-Name").unwrap());
        assert_eq!(db.inbox_id("  ").unwrap(), db.inbox_id("inbox-name").unwrap());
    }

    #[test]
    fn insert_list_read() {
        let db = Db::open_in_memory().unwrap();
        let inbox = db.inbox_id("").unwrap();
        let a = db.insert_mail(inbox, &sum("one", "one"), b"raw1", 1).unwrap();
        let b = db.insert_mail(inbox, &sum("two", "two"), b"raw2", 2).unwrap();
        let l = db.list_mails(inbox, "").unwrap();
        assert_eq!(l.iter().map(|m| m.id).collect::<Vec<_>>(), vec![b, a]);
        assert_eq!(db.list_inboxes().unwrap()[0].unread, 2);
        let m = db.get_mail(l[0].id).unwrap();
        assert_eq!(m.raw, b"raw2");
        assert_eq!(m.to, vec!["b@x.test".to_string()]);
        assert_eq!(db.list_inboxes().unwrap()[0].unread, 1);
        assert!(db.list_mails(inbox, "").unwrap()[0].is_read);
    }

    #[test]
    fn get_raw_does_not_mark_read() {
        let db = Db::open_in_memory().unwrap();
        let inbox = db.inbox_id("").unwrap();
        let id = db.insert_mail(inbox, &sum("s", "s"), b"rawbytes", 1).unwrap();
        assert_eq!(db.get_raw(id).unwrap(), b"rawbytes");
        assert_eq!(db.list_inboxes().unwrap()[0].unread, 1);
    }

    #[test]
    fn search_matches_and_escapes() {
        let db = Db::open_in_memory().unwrap();
        let inbox = db.inbox_id("").unwrap();
        db.insert_mail(inbox, &sum("s", "50% off_now"), b"r", 1).unwrap();
        assert_eq!(db.list_mails(inbox, "50%").unwrap().len(), 1);
        assert_eq!(db.list_mails(inbox, "5_%").unwrap().len(), 0);
        assert_eq!(db.list_mails(inbox, "it's").unwrap().len(), 0);
        assert_eq!(db.list_mails(inbox, "OFF").unwrap().len(), 1);
    }

    #[test]
    fn delete_mails_removes_raws() {
        let db = Db::open_in_memory().unwrap();
        let inbox = db.inbox_id("").unwrap();
        let a = db.insert_mail(inbox, &sum("a", "a"), b"r", 1).unwrap();
        let b = db.insert_mail(inbox, &sum("b", "b"), b"r", 2).unwrap();
        db.delete_mails(&[a]).unwrap();
        assert_eq!(db.list_mails(inbox, "").unwrap().iter().map(|m| m.id).collect::<Vec<_>>(), vec![b]);
        db.delete_mails(&[b]).unwrap();
        assert!(db.list_mails(inbox, "").unwrap().is_empty());
        let n: i64 = db.conn.query_row("SELECT COUNT(*) FROM raws", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn rename_rules() {
        let db = Db::open_in_memory().unwrap();
        let def = db.inbox_id("").unwrap();
        let x = db.inbox_id("x").unwrap();
        db.inbox_id("y").unwrap();
        db.rename_inbox(def, "Q").unwrap();
        assert_eq!(db.rename_inbox(x, " ").unwrap_err(), "Inbox name cannot be empty");
        assert_eq!(db.rename_inbox(x, "Y").unwrap_err(), "An inbox with this name already exists");
        db.rename_inbox(x, "Z").unwrap();
        assert!(db.list_inboxes().unwrap().iter().any(|i| i.name == "Z"));
    }

    #[test]
    fn create_rules() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.create_inbox("  ").unwrap_err(), "Inbox name cannot be empty");
        let x = db.create_inbox(" Shop ").unwrap();
        assert_eq!(db.inbox_id("shop").unwrap(), x);
        assert_eq!(db.create_inbox("SHOP").unwrap_err(), "An inbox with this name already exists");
        assert_eq!(db.create_inbox("inbox-name").unwrap_err(), "An inbox with this name already exists");
    }

    #[test]
    fn delete_inbox_cascades() {
        let db = Db::open_in_memory().unwrap();
        let x = db.inbox_id("x").unwrap();
        let m = db.insert_mail(x, &sum("a", "a"), b"r", 1).unwrap();
        db.delete_inbox(x).unwrap();
        assert!(db.get_mail(m).is_err());
        let n: i64 = db.conn.query_row("SELECT COUNT(*) FROM raws", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
        // The starter inbox is an ordinary inbox
        let def = db.inbox_id("").unwrap();
        db.delete_inbox(def).unwrap();
    }
}
