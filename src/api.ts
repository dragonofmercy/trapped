import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Inbox {
  id: number;
  name: string;
  count: number;
  unread: number;
}

export interface MailRow {
  id: number;
  subject: string;
  from: string;
  receivedAt: number;
  hasAttachments: boolean;
  isRead: boolean;
}

export interface AttachmentInfo {
  index: number;
  name: string;
  mime: string;
  size: number;
  blocked: boolean;
}

export interface MailDetail {
  id: number;
  inboxId: number;
  subject: string;
  from: string;
  to: string[];
  cc: string[];
  bcc: string[];
  date: string | null;
  receivedAt: number;
  html: string | null;
  text: string | null;
  headers: [string, string][];
  attachments: AttachmentInfo[];
}

export interface SmtpStatus {
  listening: boolean;
  address: string;
  error: string | null;
}

export const listInboxes = () => invoke<Inbox[]>("list_inboxes");
export const listMails = (inboxId: number, query: string) => invoke<MailRow[]>("list_mails", { inboxId, query });
export const getMail = (id: number) => invoke<MailDetail>("get_mail", { id });
export const getRaw = (id: number) => invoke<string>("get_raw", { id });
export const deleteMails = (ids: number[]) => invoke<void>("delete_mails", { ids });
export const renameInbox = (inboxId: number, name: string) => invoke<void>("rename_inbox", { inboxId, name });
export const createInbox = (name: string) => invoke<number>("create_inbox", { name });
export const deleteInbox = (inboxId: number) => invoke<void>("delete_inbox", { inboxId });
export const smtpStatus = () => invoke<SmtpStatus>("smtp_status");
export const smtpRetry = () => invoke<SmtpStatus>("smtp_retry");
export const getSmtpPort = () => invoke<number>("get_smtp_port");
export const checkSmtpPort = (port: number) => invoke<"current" | "free" | "busy">("check_smtp_port", { port });
export const setSmtpPort = (port: number) => invoke<SmtpStatus>("set_smtp_port", { port });
export const openAttachment = (mailId: number, index: number) => invoke<void>("open_attachment", { mailId, index });
export const saveAttachment = (mailId: number, index: number, path: string) => invoke<void>("save_attachment", { mailId, index, path });

export const onMailReceived = (cb: (inboxId: number) => void): Promise<UnlistenFn> => listen<{ inboxId: number }>("mail-received", (e) => cb(e.payload.inboxId));
export const onSmtpStatus = (cb: (s: SmtpStatus) => void): Promise<UnlistenFn> => listen<SmtpStatus>("smtp-status", (e) => cb(e.payload));

/** Starts a Tauri event subscription from a component; the returned cleanup also covers an unmount before listen() resolved */
export function subscribe(listening: Promise<UnlistenFn>): () => void {
  let dead = false;
  let unlisten: UnlistenFn | undefined;
  listening.then((u) => (dead ? u() : (unlisten = u)));
  return () => {
    dead = true;
    unlisten?.();
  };
}
