<script lang="ts">
  import { untrack } from "svelte";
  import { deleteMails, listMails, onMailReceived, subscribe, type MailRow } from "../api";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";

  let { inboxId, selectedId, refreshKey, onselect, onchanged }: { inboxId: number; selectedId: number | null; refreshKey: number; onselect: (id: number) => void; onchanged: (deletedIds: number[]) => void } = $props();

  let rows = $state<MailRow[]>([]);
  let checked = $state<number[]>([]);
  let query = $state("");
  let error = $state("");
  let deleteError = $state("");
  let confirmDelete = $state(false);

  // Only the latest request may write rows (search typing and events can overlap)
  let seq = 0;
  async function load() {
    const n = ++seq;
    try {
      const r = await listMails(inboxId, query);
      if (n !== seq) return;
      rows = r;
      const ids = new Set(r.map((m) => m.id));
      checked = checked.filter((id) => ids.has(id));
      error = "";
    } catch (e) {
      if (n === seq) error = String(e);
    }
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  function search() {
    clearTimeout(timer);
    timer = setTimeout(load, 200);
  }

  // Reload on inbox change or refresh request; the selection belongs to one inbox
  let lastInbox = -1;
  $effect(() => {
    const id = inboxId;
    void refreshKey;
    untrack(() => {
      if (id !== lastInbox) {
        lastInbox = id;
        checked = [];
        deleteError = "";
      }
      clearTimeout(timer);
      load();
    });
  });

  $effect(() => {
    const unsubscribe = subscribe(onMailReceived((id) => id === inboxId && search()));
    return () => {
      unsubscribe();
      clearTimeout(timer);
    };
  });

  const allChecked = $derived(rows.length > 0 && checked.length === rows.length);
  const someChecked = $derived(checked.length > 0 && !allChecked);
  let allBox: HTMLInputElement;
  $effect(() => {
    allBox.indeterminate = someChecked;
  });
  function toggleAll() {
    checked = allChecked ? [] : rows.map((m) => m.id);
  }

  function fmtTime(secs: number) {
    const d = new Date(secs * 1000);
    const p = (n: number) => String(n).padStart(2, "0");
    if (d.toDateString() === new Date().toDateString()) return `${p(d.getHours())}:${p(d.getMinutes())}`;
    return `${p(d.getDate())}.${p(d.getMonth() + 1)}.${d.getFullYear()}`;
  }

  async function remove() {
    const ids = [...checked];
    confirmDelete = false;
    deleteError = "";
    let ok = false;
    try {
      await deleteMails(ids);
      checked = [];
      ok = true;
    } catch (e) {
      deleteError = String(e);
    }
    await load();
    if (ok) onchanged(ids);
  }

  function open(id: number) {
    const row = rows.find((m) => m.id === id);
    if (row) row.isRead = true;
    onselect(id);
  }

  function rowKey(e: KeyboardEvent, id: number) {
    if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      open(id);
    }
  }
</script>

{#snippet check()}
  <span class="box"><svg viewBox="0 0 16 16"><path d="M3 8.5l3.2 3L13 4.5" /></svg></span>
{/snippet}

<div class="card flex min-h-0 flex-col gap-2 p-3">
  <input bind:value={query} oninput={search} class="field w-full" type="search" placeholder="Search mails" />

  <!-- Same transparent left border as the rows, so the checkboxes line up -->
  <div class="flex h-8 items-center gap-3 border-l-2 border-transparent ps-2">
    <label class="chk">
      <input type="checkbox" bind:this={allBox} checked={allChecked} disabled={!rows.length} aria-label="Select all" onchange={toggleAll} />
      {@render check()}
    </label>
    <span class="label">Inbox</span>
    {#if checked.length}
      <button type="button" class="btn danger ml-auto h-7" onclick={() => (confirmDelete = true)}><Icon name="trash" />Delete ({checked.length})</button>
    {/if}
  </div>
  {#if deleteError || error}<div class="selectable text-danger">{deleteError || error}</div>{/if}

  <div class="min-h-0 flex-1 overflow-y-auto">
    {#if !rows.length}
      {#if query}
        <p class="p-4 text-center text-muted">No mail matches.</p>
      {:else}
        <div class="flex flex-col items-center gap-2 p-8 text-muted [&_svg]:size-10"><Icon name="inbox" /><p>No email yet.</p></div>
      {/if}
    {/if}
    {#each rows as m (m.id)}
      <div
        role="button"
        tabindex="0"
        aria-label={m.subject || "(no subject)"}
        aria-current={m.id === selectedId}
        class={["flex cursor-pointer items-center gap-3 rounded-md border-l-2 px-2 py-2 hover:bg-bg2/40", m.id === selectedId ? "border-accent bg-bg2/60" : "border-transparent"]}
        onclick={() => open(m.id)}
        onkeydown={(e) => rowKey(e, m.id)}
      >
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
        <label class="chk" onclick={(e) => e.stopPropagation()}>
          <input type="checkbox" bind:group={checked} value={m.id} aria-label="Select this mail" />
          {@render check()}
        </label>
        <div class="min-w-0 flex-1">
          <div class={["truncate", m.isRead ? "text-muted" : "font-bold text-text"]}>{m.subject || "(no subject)"}</div>
          <div class="truncate text-muted">{m.from}</div>
        </div>
        {#if m.hasAttachments}<span class="flex text-muted"><Icon name="clip" /></span>{/if}
        <span class="shrink-0 font-mono text-muted">{fmtTime(m.receivedAt)}</span>
      </div>
    {/each}
  </div>
</div>

{#if confirmDelete}
  <ConfirmDialog title="Delete Mails" message={`Are you sure you want to delete the ${checked.length === 1 ? "selected mail" : `${checked.length} selected mails`}?`} confirmLabel="Delete" onconfirm={remove} oncancel={() => (confirmDelete = false)} />
{/if}
