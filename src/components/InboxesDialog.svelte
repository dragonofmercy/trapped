<script lang="ts">
  import { createInbox, deleteInbox, renameInbox, type Inbox } from "../api";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";
  import NameDialog from "./NameDialog.svelte";

  let { inboxes, onchanged, onclose }: { inboxes: Inbox[]; onchanged: (selectId?: number) => void; onclose: () => void } = $props();

  // Name dialog for New and Rename; inbox = null means New
  type Naming = { inbox: Inbox | null; error: string };
  let naming = $state<Naming | null>(null);
  let deleting = $state<Inbox | null>(null);
  let error = $state("");
  let query = $state("");
  const shown = $derived(inboxes.filter((i) => i.name.toLowerCase().includes(query.trim().toLowerCase())));

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

<Modal title="Inboxes" class="w-150" {onclose}>
  <div class="flex h-100 flex-col">
    <!-- Search on the left, add on the right -->
    <div class="flex items-center gap-2">
      <label class="field search">
        <Icon name="search" />
        <input type="search" bind:value={query} placeholder="Search inboxes..." aria-label="Search inboxes" />
      </label>
      <button type="button" class="btn" onclick={() => (naming = { inbox: null, error: "" })}><Icon name="mailplus" />Add inbox</button>
    </div>
    {#if error}<p class="error selectable mt-3">{error}</p>{/if}

    <!-- The scroller overhangs the card by 1px, hidden by overflow: scrolled to the end, the last row's line sits under the card border -->
    <div class="list mt-3 flex min-h-0 flex-1 flex-col overflow-hidden">
      <div class="-mb-px min-h-0 flex-1 overflow-y-auto">
        {#if !inboxes.length}
          <p class="py-4 text-center text-muted">No inbox. One is created on the first mail sent with a new username.</p>
        {:else if !shown.length}
          <p class="py-4 text-center text-muted">No inbox matches.</p>
        {/if}
        {#each shown as i (i.id)}
          <div class="item">
            <span class="tile"><Icon name="inbox" /></span>
            <span class="text">
              <strong>{i.name}</strong>
              <span><b>{i.count}</b> {i.count === 1 ? "mail" : "mails"}{#if i.unread}<span class="sep"></span><b>{i.unread}</b> unread{/if}</span>
            </span>
            <button type="button" class="btn icon tip-left" data-tip="Rename" aria-label={`Rename ${i.name}`} onclick={() => (naming = { inbox: i, error: "" })}><Icon name="pencil" /></button>
            <button type="button" class="btn danger icon tip-left" data-tip="Delete" aria-label={`Delete ${i.name}`} onclick={() => (deleting = i)}><Icon name="trash" /></button>
          </div>
        {/each}
      </div>
    </div>
  </div>
</Modal>

{#if naming}
  {@const n = naming}
  <NameDialog
    title={n.inbox ? "Rename Inbox" : "Add Inbox"}
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
    message={`Are you sure you want to delete the inbox "${deleting.name}"? Its ${deleting.count} ${deleting.count === 1 ? "mail" : "mails"} will be deleted as well.`}
    confirmLabel="Delete"
    onconfirm={confirmDelete}
    oncancel={() => (deleting = null)}
  />
{/if}

<style>
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 2.5);
    padding: 0 calc(var(--spacing) * 3);
    color: var(--muted);
    cursor: text;
  }
  .search:focus-within {
    border-color: var(--cyan);
    box-shadow: 0 0 0 3px rgba(var(--cyan-rgb), 0.15);
  }
  .search input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--text);
    font: inherit;
  }
  /* One card holding the list, rows separated by a border */
  .list {
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: linear-gradient(180deg, var(--card-a), var(--card-b));
  }
  .item {
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 3);
    padding: calc(var(--spacing) * 2.5) calc(var(--spacing) * 3);
  }
  /* Every row, the last one included: the list keeps its height, so the last row needs a closing line too */
  .item {
    border-bottom: 1px solid var(--line);
  }
  .tile {
    display: flex;
    align-items: center;
    justify-content: center;
    width: calc(var(--spacing) * 8.5);
    height: calc(var(--spacing) * 8.5);
    flex-shrink: 0;
    border-radius: var(--radius-lg);
    color: var(--cyan);
    background: rgba(var(--accent-rgb), 0.12);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: calc(var(--spacing) * 0.75);
  }
  .text strong {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .text > span {
    color: var(--muted);
  }
  .text b {
    font-weight: 600;
    color: var(--accent-hi);
  }
  .sep {
    display: inline-block;
    width: calc(var(--spacing) * 3.5);
  }
</style>
