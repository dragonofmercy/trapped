<script lang="ts">
  import { onMount } from "svelte";
  import { getSmtpPort, setSmtpPort } from "../api";
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

<Modal title="Options" width={100} {onclose}>
  <form class="form" onsubmit={submit}>
    <label class="form-row">
      <span class="text-muted">SMTP port</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input class="field w-full" inputmode="numeric" bind:value={port} autofocus oninput={() => (error = "")} />
      {#if invalid || error}<p class="error selectable">{invalid || error}</p>{/if}
    </label>
    <div class="buttons">
      <button type="button" class="btn ghost" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn" disabled={saving || invalid !== null || port.trim() === ""}>Save</button>
    </div>
  </form>
</Modal>
