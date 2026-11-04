<script lang="ts">
  import { onMount } from "svelte";
  import { getSmtpPort, setSmtpPort } from "../api";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let port = $state("");
  let error = $state("");
  let saving = $state(false);
  onMount(() => {
    getSmtpPort().then(
      (p) => (port = String(p)),
      (e) => (error = String(e)),
    );
  });

  const value = $derived(Number(port.trim()));
  const invalid = $derived(port.trim() !== "" && !(Number.isInteger(value) && value >= 1 && value <= 65535) ? "Port must be a number between 1 and 65535" : null);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (invalid !== null || port.trim() === "") return;
    saving = true;
    error = "";
    try {
      await setSmtpPort(value);
      onclose();
    } catch (err) {
      error = String(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal title="Options" width={150} {onclose}>
  <form class="form" onsubmit={submit}>
    <!-- Windows 11 settings layout: section heading, then one card per setting (icon, title + description, control on the right) -->
    <section class="space-y-2">
      <h3 class="text-muted font-semibold">SMTP server</h3>
      <div class="setting">
        <span class="tile"><Icon name="plug" /></span>
        <label for="smtp-port" class="min-w-0 flex-1">
          <span class="block font-semibold">Port</span>
          <span class="block text-muted">Port Trapped listens on, on 127.0.0.1 only. Default 2525.</span>
        </label>
        <!-- svelte-ignore a11y_autofocus -->
        <input id="smtp-port" class="field w-28 text-right" inputmode="numeric" bind:value={port} autofocus oninput={() => (error = "")} />
      </div>
      {#if invalid || error}<p class="error selectable">{invalid || error}</p>{/if}
    </section>
    <div class="buttons">
      <button type="button" class="btn ghost" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn" disabled={saving || invalid !== null || port.trim() === ""}>Save</button>
    </div>
  </form>
</Modal>

<style>
  .setting {
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 4);
    padding: calc(var(--spacing) * 3.5) calc(var(--spacing) * 4);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: linear-gradient(180deg, var(--card-a), var(--card-b));
  }
  .tile {
    display: flex;
    align-items: center;
    justify-content: center;
    width: calc(var(--spacing) * 8.5);
    height: calc(var(--spacing) * 8.5);
    flex-shrink: 0;
    border-radius: var(--radius-md);
    color: var(--cyan);
    background: rgba(var(--accent-rgb), 0.12);
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
</style>
