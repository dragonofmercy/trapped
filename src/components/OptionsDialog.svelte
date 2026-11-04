<script lang="ts">
  import { onMount } from "svelte";
  import { checkSmtpPort, getSmtpPort, setSmtpPort } from "../api";
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

  // Availability of the typed port, checked 300ms after the last keystroke; null while unknown
  let availability = $state<{ port: number; state: "current" | "free" | "busy" | "error"; message?: string } | null>(null);
  $effect(() => {
    const p = port;
    availability = null;
    if (p === null || invalid !== null) return;
    const timer = setTimeout(() => {
      checkSmtpPort(p).then(
        (state) => port === p && (availability = { port: p, state }),
        (e) => port === p && (availability = { port: p, state: "error", message: String(e) }),
      );
    }, 300);
    return () => clearTimeout(timer);
  });
  const dot = $derived(
    availability?.state === "current"
      ? { color: "bg-accent", tip: "Trapped is listening on this port" }
      : availability?.state === "free"
        ? { color: "bg-ok", tip: "Port is available" }
        : availability?.state === "busy"
          ? { color: "bg-danger", tip: "Port is already in use by another program" }
          : availability?.state === "error"
            ? { color: "bg-danger", tip: availability.message ?? "Port cannot be used" }
            : { color: "bg-dim", tip: "Checking port" },
  );
  const busy = $derived(availability?.state === "busy" || availability?.state === "error");

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
        <!-- Availability dot: blue = Trapped's current port, green = free, red = in use or refused, grey = not checked yet -->
        <span class="flex items-center gap-2">
          <span class="size-2 shrink-0 rounded-full {dot.color}" data-tip={dot.tip} role="img" aria-label={dot.tip}></span>
          <!-- svelte-ignore a11y_autofocus -->
          <input class="field w-28 text-right" type="number" min="1" max="65535" autocomplete="off" bind:value={port} autofocus oninput={() => (error = "")} />
        </span>
      </label>
      {#if invalid || error}<p class="error selectable">{invalid || error}</p>{/if}
    </div>
    <!-- Setup help: the snippet follows the port being typed while it is valid -->
    <div class="help space-y-2">
      <p>Mails are grouped by SMTP username: pick any name, Trapped creates the inbox on the first mail it receives.</p>
      <p>Laravel <code>.env</code>:</p>
      <pre class="selectable">MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT={invalid === null && port !== null ? port : 2525}
MAIL_USERNAME=Inbox-Name
MAIL_PASSWORD=null
MAIL_ENCRYPTION=null</pre>
    </div>
    <div class="buttons">
      <button type="button" class="btn ghost" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn" disabled={saving || invalid !== null || port === null || busy}>Save</button>
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
  .help {
    color: var(--muted);
  }
  .help code,
  .help pre {
    font-family: var(--font-mono);
    color: var(--text);
    background: var(--field);
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
  }
  .help code {
    padding: 0 var(--spacing);
  }
  .help pre {
    margin: 0;
    padding: calc(var(--spacing) * 3) calc(var(--spacing) * 4);
    line-height: 1.6;
  }
  .error {
    margin: 0;
    color: var(--danger);
  }
</style>
