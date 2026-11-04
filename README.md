# Trapped

Desktop mail catcher (Tauri 2 + Svelte 5). It listens for SMTP on `127.0.0.1:2525` (port and listen address configurable in Options; `0.0.0.0` accepts mail from other machines, then use this machine's IP as `MAIL_HOST`), accepts every message, never relays, and shows what it caught. Closing the window hides it to the tray.

## Laravel setup

```
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=2525
MAIL_USERNAME=Inbox-Name
MAIL_PASSWORD=null
MAIL_ENCRYPTION=null
```

## Inboxes

The SMTP AUTH username is the inbox name (any password is accepted). An inbox is created on its first mail. The app starts with an `Inbox-Name` inbox; every inbox can be renamed or deleted from Options. Mail sent without AUTH goes to `Inbox-Name` (recreated if needed). No TLS.

## Development

```
npm install
npm run tauri dev
npm run tauri build
```
