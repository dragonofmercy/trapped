import { onSmtpStatus, smtpRetry, smtpStatus, type SmtpStatus } from "./api";

// One app-wide subscription (lives as long as the window), read by the top bar and the empty mail list
export const smtp = $state<{ status: SmtpStatus | null }>({ status: null });

const failed = (e: unknown): SmtpStatus => ({ listening: false, address: "", error: String(e) });

smtpStatus().then(
  (s) => (smtp.status = s),
  (e) => (smtp.status = failed(e)),
);
onSmtpStatus((s) => (smtp.status = s));

export async function retrySmtp() {
  try {
    smtp.status = await smtpRetry();
  } catch (e) {
    smtp.status = failed(e);
  }
}
