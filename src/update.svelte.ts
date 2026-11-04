import { check, type Update } from "@tauri-apps/plugin-updater";
import { getCheckUpdates } from "./api";

// The pending update, shown as a banner under the top bar until installed or dismissed
export const updater = $state<{ update: Update | null }>({ update: null });

/** Looks for a new version when the option is on. Offline or GitHub unreachable: no banner, no error. */
export async function checkUpdate() {
  if (!(await getCheckUpdates().catch(() => true))) return;
  updater.update = await check({ timeout: 10000 }).catch(() => null);
}
