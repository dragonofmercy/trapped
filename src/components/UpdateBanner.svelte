<script lang="ts">
  import type { Update } from "@tauri-apps/plugin-updater";
  import { updater } from "../update.svelte";
  import Icon from "./Icon.svelte";

  let { update }: { update: Update } = $props();
  let progress = $state<number | null>(null);
  let error = $state("");

  /** Downloads, checks the signature, then runs the installer: Windows closes Trapped and the installer restarts it. */
  async function install() {
    let total = 0;
    let done = 0;
    progress = 0;
    error = "";
    try {
      await update.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        if (e.event === "Progress") done += e.data.chunkLength;
        if (total) progress = Math.round((done / total) * 100);
      });
    } catch (e) {
      progress = null;
      error = `Update failed: ${e}`;
    }
  }
</script>

<div class="update">
  <span class="ico"><Icon name="download" /></span>
  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
    <strong class="text-text">Trapped {update.version} is available</strong>
    {#if error}<span class="selectable text-danger">{error}</span>{:else}<span class="text-muted">You have {update.currentVersion}. Trapped will restart to finish the update.</span>{/if}
  </div>
  <button type="button" class="btn ghost" disabled={progress !== null} onclick={() => (updater.update = null)}>Later</button>
  <button type="button" class="btn" disabled={progress !== null} onclick={install}><Icon name="download" />{progress === null ? "Install update" : `Downloading ${progress}%`}</button>
</div>

<style>
  .update {
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 3);
    padding: calc(var(--spacing) * 2) calc(var(--spacing) * 2) calc(var(--spacing) * 2) calc(var(--spacing) * 3);
    border-radius: var(--radius-xl);
    border: 1px solid rgba(var(--cyan-rgb), 0.45);
    background: linear-gradient(90deg, rgba(var(--cyan-rgb), 0.14), rgba(var(--accent-rgb), 0.06));
  }
  .ico {
    display: grid;
    place-items: center;
    width: calc(var(--spacing) * 8);
    height: calc(var(--spacing) * 8);
    border-radius: 50%;
    color: var(--cyan);
    background: rgba(var(--cyan-rgb), 0.14);
    border: 1px solid rgba(var(--cyan-rgb), 0.35);
  }
</style>
