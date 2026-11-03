<script lang="ts">
  import { onMount } from "svelte";
  import { createInbox, deleteInbox, renameInbox, type Inbox } from "../api";
  import { getTheme, setTheme, type Theme } from "../theme";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import NameDialog from "./NameDialog.svelte";

  let { inboxes, onchanged, onclose }: { inboxes: Inbox[]; onchanged: (selectId?: number) => void; onclose: () => void } = $props();

  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());

  let section = $state<"general" | "inboxes">("general");

  const THEMES: { value: Theme; label: string }[] = [
    { value: "system", label: "Follow Windows setting" },
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
  ];
  let theme = $state(getTheme());

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
  <div class="flex h-[440px]">
    <nav class="flex w-44 flex-col gap-1 border-r border-line p-3">
      <h2 class="px-2 pt-1">Options</h2>
      <button type="button" class="nav" class:on={section === "general"} onclick={() => (section = "general")}><Icon name="sliders" />General</button>
      <button type="button" class="nav" class:on={section === "inboxes"} onclick={() => (section = "inboxes")}><Icon name="inbox" />Inboxes</button>
    </nav>

    <section class="flex min-w-0 flex-1 flex-col p-[18px]">
      {#if section === "general"}
        <h2>Theme</h2>
        <div class="flex flex-col gap-[9px]">
          {#each THEMES as t (t.value)}
            <label class="chk radio"><input type="radio" name="theme" value={t.value} bind:group={theme} onchange={() => setTheme(theme)} /><span class="box"></span>{t.label}</label>
          {/each}
        </div>
      {:else}
        <div class="flex items-center">
          <h2 class="m-0">Inboxes</h2>
          <button type="button" class="btn ml-auto" onclick={() => (naming = { inbox: null, error: "" })}><Icon name="plus" />New</button>
        </div>
        {#if error}<p class="error selectable">{error}</p>{/if}
        <ul class="m-0 mt-3 flex min-h-0 flex-1 list-none flex-col gap-1.5 overflow-y-auto p-0">
          {#each inboxes as i (i.id)}
            <li class="flex h-11 items-center gap-2 rounded-lg border border-line px-3">
              <span class="flex text-cyan"><Icon name="inbox" /></span>
              <span class="min-w-0 flex-1 truncate font-medium">{i.name}</span>
              <span class="font-mono text-muted">{i.count}</span>
              {#if i.name !== "Default"}
                <button type="button" class="btn ghost h-8 px-2.5" title="Rename" aria-label={`Rename ${i.name}`} onclick={() => (naming = { inbox: i, error: "" })}><Icon name="pencil" /></button>
                <button type="button" class="btn danger h-8 px-2.5" title="Delete" aria-label={`Delete ${i.name}`} onclick={() => (deleting = i)}><Icon name="trash" /></button>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}

      <div class="buttons mt-auto pt-[18px]">
        <button type="button" class="btn ghost" onclick={onclose}>Close</button>
      </div>
    </section>
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
  dialog {
    width: 720px;
    max-width: 90vw;
    padding: 0;
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    font:
      500 13px Inter,
      sans-serif;
    cursor: pointer;
    text-align: left;
  }
  .nav:hover {
    background: rgba(var(--accent-rgb), 0.1);
    color: var(--text);
  }
  .nav.on {
    background: rgba(var(--accent-rgb), 0.24);
    color: var(--text);
  }
</style>
