<script lang="ts">
  import { onMount } from "svelte";
  import { emptyInbox, onSmtpStatus, smtpRetry, smtpStatus, type Inbox, type SmtpStatus } from "../api";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
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

  // An error belongs to the inbox it happened on
  let actionError = $state("");
  $effect(() => {
    void currentInboxId;
    actionError = "";
  });

  let confirmEmpty = $state(false);
  async function empty() {
    confirmEmpty = false;
    actionError = "";
    try {
      await emptyInbox(currentInboxId);
      onchanged(currentInboxId);
    } catch (e) {
      actionError = String(e);
      onchanged();
    }
  }
</script>

{#snippet inboxIcon()}
  <span class="flex text-cyan"><Icon name="inbox" /></span>
{/snippet}

<div class="card flex items-center gap-3 py-2.5">
  <span class="label">Inbox</span>
  <div class="w-72"><Select label="Inbox" bind:value={() => currentInboxId, (v) => onselect(v)} {options} icon={inboxIcon} /></div>
  <button type="button" class="btn warn" disabled={!current?.count} onclick={() => (confirmEmpty = true)}><Icon name="trash" />Empty</button>
  {#if actionError}<span class="selectable text-danger">{actionError}</span>{/if}

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

{#if confirmEmpty}
  <ConfirmDialog title="Empty Inbox" message={`Delete all mails in "${current?.name}"?`} confirmLabel="Empty" onconfirm={empty} oncancel={() => (confirmEmpty = false)} />
{/if}
