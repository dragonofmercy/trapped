import { getCurrentWindow } from "@tauri-apps/api/window";
import { ref, watch } from "vue";

export type Theme = "system" | "dark" | "light";

const KEY = "theme";

function load(): Theme {
  try {
    const t = localStorage.getItem(KEY);
    if (t === "dark" || t === "light") return t;
  } catch {
    // Storage unavailable: fall back to the system setting
  }
  return "system";
}

export const theme = ref<Theme>(load());

const media = matchMedia("(prefers-color-scheme: light)");

function apply() {
  const light = theme.value === "light" || (theme.value === "system" && media.matches);
  document.documentElement.dataset.theme = light ? "light" : "dark";
}

media.addEventListener("change", apply);

watch(
  theme,
  (t) => {
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
  },
  { immediate: true },
);
