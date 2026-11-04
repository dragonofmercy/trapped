<script lang="ts">
  import { onMount } from "svelte";
  import { getSmtpPort, setSmtpPort } from "../api";
  import Modal from "./Modal.svelte";

  let { onclose }: { onclose: () => void } = $props();

  // Number input: bind:value gives null while the field is empty
  let port = $state<number | null>(null);
  let error = $state("");
  let saving = $state(false);
  onMount(() => {
    getSmtpPort().then(
      (p) => (port = p),
      (e) => (error = String(e)),
    );
  });

  const invalid = $derived(port !== null && !(Number.isInteger(port) && port >= 1 && port <= 65535) ? "Port must be a number between 1 and 65535" : null);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (invalid !== null || port === null) return;
    saving = true;
    error = "";
    try {
      await setSmtpPort(port);
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
    <!-- Wallo options layout: one row per option (title + description, control on the right), rows separated by a border -->
    <div>
      <label class="option">
        <span class="text"><strong>SMTP port</strong><span>Port Trapped listens on, on 127.0.0.1 only. Default 2525.</span></span>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="field w-28 text-right" type="number" min="1" max="65535" autocomplete="off" bind:value={port} autofocus oninput={() => (error = "")} />
      </label>
      {#if invalid || error}<p class="error selectable">{invalid || error}</p>{/if}
    </div>
    <div class="buttons">
      <button type="button" class="btn ghost" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn" disabled={saving || invalid !== null || port === null}>Save</button>
    </div>
  </form>
</Modal>

<style>
  .option {
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 6);
    padding: calc(var(--spacing) * 3.5) 0;
  }
  .option:not(:first-child) {
    border-top: 1px solid var(--line);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing);
  }
  .text strong {
    font-weight: 600;
  }
  .text span {
    color: var(--muted);
  }
  /* No spin buttons on the number field */
  input::-webkit-inner-spin-button,
  input::-webkit-outer-spin-button {
    appearance: none;
    margin: 0;
  }
  input {
    appearance: textfield;
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
</style>
