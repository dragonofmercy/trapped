import { getCurrentWindow } from "@tauri-apps/api/window";

export type Theme = "system" | "dark" | "light";

const KEY = "theme";
const media = matchMedia("(prefers-color-scheme: light)");

export function getTheme(): Theme {
  try {
    const t = localStorage.getItem(KEY);
    if (t === "dark" || t === "light") return t;
  } catch {
    // Storage unavailable: fall back to the system setting
  }
  return "system";
}

function apply() {
  const t = getTheme();
  const light = t === "light" || (t === "system" && media.matches);
  document.documentElement.dataset.theme = light ? "light" : "dark";
}

export function setTheme(t: Theme) {
  try {
    localStorage.setItem(KEY, t);
  } catch {
    // Not persisted, still applied for this session
  }
  apply();
  // Native title bar follows the choice too
  getCurrentWindow()
    .setTheme(t === "system" ? null : t)
    .catch(() => {});
}

media.addEventListener("change", apply);
setTheme(getTheme());
