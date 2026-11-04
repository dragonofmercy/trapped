mod commands;
mod db;
mod parse;
mod smtp;

use commands::{DbState, SmtpState, SmtpTx};
use serde::Serialize;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_window_state::StateFlags;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MailReceived {
    inbox_id: i64,
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn store(app: &AppHandle, r: smtp::Received) {
    let summary = parse::summarize(&r.raw, &r.rcpt_to);
    let db = app.state::<DbState>();
    let db = commands::lock(&db);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    match db.inbox_id(&r.inbox).and_then(|id| db.insert_mail(id, &summary, &r.raw, now).map(|_| id)) {
        Ok(inbox_id) => {
            let _ = app.emit("mail-received", MailReceived { inbox_id });
        }
        Err(err) => eprintln!("Failed to store received mail: {err}"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // Position, size and maximized state survive restarts; visibility is left out, else quitting from the tray would start hidden
        .plugin(tauri_plugin_window_state::Builder::default().with_state_flags(StateFlags::all() & !StateFlags::VISIBLE).build())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data)?;
            let db = db::Db::open(&data.join("trapped.db"))?;
            let port = db.smtp_port()?;
            app.manage::<DbState>(Mutex::new(db));

            let attachments = app.path().app_cache_dir()?.join("attachments");
            let _ = std::fs::remove_dir_all(&attachments);
            std::fs::create_dir_all(&attachments)?;

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let smtp = tauri::async_runtime::block_on(commands::Smtp::start(tx.clone(), port));
            app.manage::<SmtpState>(tokio::sync::Mutex::new(smtp));
            app.manage(SmtpTx(tx));
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(r) = rx.recv().await {
                    store(&handle, r);
                }
            });

            let menu = Menu::with_items(app, &[&MenuItem::with_id(app, "show", "Show", true, None::<&str>)?, &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().ok_or("missing window icon")?)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, ev| {
            if let WindowEvent::CloseRequested { api, .. } = ev {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_inboxes,
            commands::list_mails,
            commands::get_mail,
            commands::get_raw,
            commands::delete_mails,
            commands::rename_inbox,
            commands::create_inbox,
            commands::get_smtp_port,
            commands::set_smtp_port,
            commands::check_smtp_port,
            commands::delete_inbox,
            commands::smtp_status,
            commands::smtp_retry,
            commands::open_attachment,
            commands::save_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
