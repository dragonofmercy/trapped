<script lang="ts">
  import { onMount } from "svelte";
  import { createInbox, deleteInbox, renameInbox, type Inbox } from "../api";
  import { getTheme, setTheme, type Theme } from "../theme";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";
  import NameDialog from "./NameDialog.svelte";
  import Select from "./Select.svelte";

  let { inboxes, onchanged, onclose }: { inboxes: Inbox[]; onchanged: (selectId?: number) => void; onclose: () => void } = $props();

  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());

  let section = $state<"general" | "inboxes">("general");

  const THEMES: { value: Theme; label: string }[] = [
    { value: "system", label: "System" },
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
    <!-- Both columns: same top padding and a 34px title row, so the two titles line up -->
    <nav class="flex w-44 flex-col gap-1 border-r border-line px-3 pt-[18px] pb-3">
      <h2 class="m-0 flex h-[34px] items-center px-2">Options</h2>
      <button type="button" class="nav" class:on={section === "general"} onclick={() => (section = "general")}><Icon name="sliders" />General</button>
      <button type="button" class="nav" class:on={section === "inboxes"} onclick={() => (section = "inboxes")}><Icon name="inbox" />Inboxes</button>
    </nav>

    <section class="flex min-w-0 flex-1 flex-col p-[18px]">
      <!-- Page title row, shared by every page, ends with the close cross -->
      <div class="page flex items-center gap-2">
        <h2 class="m-0 mr-auto">{section === "general" ? "General" : "Inboxes"}</h2>
        {#if section === "inboxes"}
          <button type="button" class="btn" onclick={() => (naming = { inbox: null, error: "" })}><Icon name="plus" />New</button>
        {/if}
        <button type="button" class="btn ghost close" title="Close" aria-label="Close" onclick={onclose}><Icon name="close" /></button>
      </div>
      {#if section === "general"}
        <div class="option">
          <span class="text"><strong>Theme</strong><span>Dark or light interface, or follow the Windows setting.</span></span>
          <div class="w-40"><Select label="Theme" bind:value={() => theme, (v) => ((theme = v), setTheme(v))} options={THEMES} /></div>
        </div>
      {:else}
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
      {/if}

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
  /* Page title on top, then one option per row (Wallo's options layout) */
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
