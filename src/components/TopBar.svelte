<script lang="ts">
  import { onMount } from "svelte";
  import { onSmtpStatus, smtpRetry, smtpStatus, type Inbox, type SmtpStatus } from "../api";
  import Icon from "./Icon.svelte";
  import Select from "./Select.svelte";

  let { inboxes, currentInboxId, onselect, onmanage }: { inboxes: Inbox[]; currentInboxId: number; onselect: (id: number) => void; onmanage: () => void } = $props();

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
</script>

{#snippet inboxIcon()}
  <span class="flex text-cyan"><Icon name="inbox" /></span>
{/snippet}

<div class="card flex items-center gap-3 p-3">
  <div class="w-72"><Select label="Inbox" bind:value={() => currentInboxId, (v) => onselect(v)} {options} icon={inboxIcon} /></div>
  <button type="button" class="btn w-[34px] justify-center p-0" title="Manage inboxes" aria-label="Manage inboxes" onclick={onmanage}><Icon name="gear" /></button>

  <div class="ml-auto flex items-center gap-2">
    {#if status?.listening}
      <span class="size-2 rounded-full bg-ok"></span>
      <span class="font-mono text-muted">{status.address}</span>
    {:else if status}
      <span class="size-2 rounded-full bg-danger"></span>
      <span class="selectable text-danger">{status.error ?? "SMTP server not running"}</span>
      <button type="button" class="btn" disabled={retrying} onclick={retry}><Icon name="reset" />Retry</button>
    {/if}
  </div>
</div>
