<script lang="ts">
  import { onMount } from "svelte";
  import { onSmtpStatus, renameInbox, smtpRetry, smtpStatus, type Inbox, type SmtpStatus } from "../api";
  import Icon from "./Icon.svelte";
  import NameDialog from "./NameDialog.svelte";
  import Select from "./Select.svelte";

  let { inboxes, currentInboxId, onselect, onchanged, onoptions }: { inboxes: Inbox[]; currentInboxId: number; onselect: (id: number) => void; onchanged: (selectId?: number) => void; onoptions: () => void } = $props();

  const current = $derived(inboxes.find((i) => i.id === currentInboxId));
  const options = $derived(inboxes.map((i) => ({ value: i.id, label: i.name, badge: i.count })));

  let status = $state<SmtpStatus | null>(null);
  onMount(() => {
    let unlisten: (() => void) | undefined;
    let dead = false;
    onSmtpStatus((s) => (status = s)).then((u) => (dead ? u() : (unlisten = u)));
    smtpStatus().then(
      (s) => (status = s),
      (e) => (status = { listening: false, address: "", error: String(e) }),
    );
    return () => {
      dead = true;
      unlisten?.();
    };
  });

  let retrying = $state(false);
  async function retry() {
    retrying = true;
    try {
      status = await smtpRetry();
    } catch (e) {
      status = { listening: false, address: "", error: String(e) };
    } finally {
      retrying = false;
    }
  }

  // Edit renames the current inbox; backend errors stay in the dialog
  let editing = $state<{ error: string } | null>(null);
  function nameError(name: string) {
    const n = name.trim();
    if (!n) return "";
    if (inboxes.some((i) => i.id !== currentInboxId && i.name.toLowerCase() === n.toLowerCase())) return "An inbox with this name already exists";
    return null;
  }
  async function rename(name: string) {
    const e = editing;
    if (!e) return;
    try {
      await renameInbox(currentInboxId, name);
      editing = null;
      onchanged(currentInboxId);
    } catch (err) {
      e.error = String(err);
    }
  }
</script>

{#snippet inboxIcon()}
  <span class="flex text-cyan"><Icon name="inbox" /></span>
{/snippet}

<div class="card flex items-center gap-3 p-3">
  <div class="w-72"><Select label="Inbox" bind:value={() => currentInboxId, (v) => onselect(v)} {options} icon={inboxIcon} /></div>
  <button type="button" class="btn" disabled={!current} onclick={() => (editing = { error: "" })}><Icon name="pencil" />Edit</button>

  <div class="ml-auto flex items-center gap-2">
    {#if status?.listening}
      <span class="size-2 rounded-full bg-ok"></span>
      <span class="font-mono text-muted">{status.address}</span>
    {:else if status}
      <span class="size-2 rounded-full bg-danger"></span>
      <span class="selectable text-danger">{status.error ?? "SMTP server not running"}</span>
      <button type="button" class="btn" disabled={retrying} onclick={retry}><Icon name="reset" />Retry</button>
    {/if}
    <button type="button" class="btn ghost" onclick={onoptions}><Icon name="sliders" />Options</button>
  </div>
</div>

{#if editing && current}
  <NameDialog title="Rename Inbox" initial={current.name} validate={nameError} error={editing.error} onconfirm={rename} oncancel={() => (editing = null)} />
{/if}
