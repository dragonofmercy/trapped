<script lang="ts">
  import { onMount } from "svelte";
  import { createInbox, deleteInbox, renameInbox, type Inbox } from "../api";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import NameDialog from "./NameDialog.svelte";

  let { inboxes, onchanged, onclose }: { inboxes: Inbox[]; onchanged: (selectId?: number) => void; onclose: () => void } = $props();

  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());

  // Name dialog for New and Rename; inbox = null means New
  type Naming = { inbox: Inbox | null; error: string };
  let naming = $state<Naming | null>(null);
  let deleting = $state<Inbox | null>(null);
  let error = $state("");

  function nameError(name: string, except?: Inbox | null) {
    const n = name.trim();
    if (!n) return "";
    if (inboxes.some((i) => i.id !== except?.id && i.name.toLowerCase() === n.toLowerCase())) return "An inbox with this name already exists";
    return null;
  }

  async function confirmName(name: string) {
    const current = naming;
    if (!current) return;
    const target = current.inbox;
    try {
      if (target) {
        await renameInbox(target.id, name);
        onchanged();
      } else {
        onchanged(await createInbox(name));
      }
      naming = null;
    } catch (e) {
      current.error = String(e);
    }
  }

  async function confirmDelete() {
    const target = deleting;
    deleting = null;
    error = "";
    if (!target) return;
    try {
      await deleteInbox(target.id);
    } catch (e) {
      error = String(e);
    }
    onchanged();
  }
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), onclose())}>
  <div class="flex h-[440px] flex-col">
    <div class="page flex items-center gap-2">
      <h2 class="m-0 mr-auto">Inboxes</h2>
      <button type="button" class="btn" onclick={() => (naming = { inbox: null, error: "" })}><Icon name="plus" />New</button>
      <button type="button" class="btn ghost close" title="Close" aria-label="Close" onclick={onclose}><Icon name="close" /></button>
    </div>
    {#if error}<p class="error selectable">{error}</p>{/if}
    <div class="min-h-0 flex-1 overflow-y-auto">
      {#if !inboxes.length}<p class="m-0 py-4 text-muted">No inbox. One is created on the first mail sent with a new username.</p>{/if}
      {#each inboxes as i (i.id)}
        <div class="option">
          <span class="text"><strong>{i.name}</strong><span>{i.count} {i.count === 1 ? "mail" : "mails"}</span></span>
          <div class="flex gap-2">
            <button type="button" class="btn" onclick={() => (naming = { inbox: i, error: "" })}><Icon name="pencil" />Rename</button>
            <button type="button" class="btn danger" onclick={() => (deleting = i)}><Icon name="trash" />Delete</button>
          </div>
        </div>
      {/each}
    </div>
  </div>
</dialog>

{#if naming}
  {@const n = naming}
  <NameDialog
    title={n.inbox ? "Rename Inbox" : "New Inbox"}
    initial={n.inbox?.name ?? ""}
    validate={(name) => nameError(name, n.inbox)}
    error={n.error}
    onconfirm={confirmName}
    oncancel={() => (naming = null)}
  />
{/if}

{#if deleting}
  <ConfirmDialog
    title="Delete Inbox"
    message={`Delete inbox "${deleting.name}" and its ${deleting.count} ${deleting.count === 1 ? "mail" : "mails"}? This cannot be undone.`}
    confirmLabel="Delete"
    onconfirm={confirmDelete}
    oncancel={() => (deleting = null)}
  />
{/if}

<style>
  /* Title row on top, then one inbox per row (Wallo's options layout) */
  .page {
    height: 34px;
    margin-bottom: 4px;
  }
  .close {
    width: 34px;
    padding: 0;
    justify-content: center;
    border-color: transparent;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 14px 0;
  }
  .option + .option {
    border-top: 1px solid var(--line);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .text strong {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .text span {
    color: var(--muted);
  }
  dialog {
    width: 560px;
    max-width: 90vw;
  }
</style>
