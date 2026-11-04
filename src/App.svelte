<script lang="ts">
  import { onMount } from "svelte";
  import { listInboxes, onMailReceived, subscribe, type Inbox } from "./api";
  import MailList from "./components/MailList.svelte";
  import MailView from "./components/MailView.svelte";
  import InboxesDialog from "./components/InboxesDialog.svelte";
  import OptionsDialog from "./components/OptionsDialog.svelte";
  import TopBar from "./components/TopBar.svelte";

  let inboxes = $state<Inbox[]>([]);
  let currentInboxId = $state(0);
  let selectedMailId = $state<number | null>(null);
  let listKey = $state(0);
  let showInboxes = $state(false);
  let showOptions = $state(false);

  function select(id: number) {
    currentInboxId = id;
    selectedMailId = null;
  }

  async function reload(selectId?: number) {
    try {
      inboxes = await listInboxes();
    } catch {
      return;
    }
    if (selectId !== undefined) select(selectId);
    else if (!inboxes.some((i) => i.id === currentInboxId)) select(inboxes[0]?.id ?? 0);
  }

  onMount(() => {
    reload();
    return subscribe(onMailReceived(reloadSoon));
  });

  // A burst of incoming mails (a test suite) reloads the inbox counts once
  let reloadTimer: ReturnType<typeof setTimeout> | undefined;
  function reloadSoon() {
    clearTimeout(reloadTimer);
    reloadTimer = setTimeout(reload, 150);
  }

  function refresh() {
    listKey++;
    reload();
  }

  function inboxesChanged(selectId?: number) {
    listKey++;
    selectedMailId = null;
    reload(selectId);
  }

  function listChanged(deletedIds: number[]) {
    if (selectedMailId !== null && deletedIds.includes(selectedMailId)) selectedMailId = null;
    reload();
  }
</script>

<div class="flex h-full flex-col gap-3 p-3">
  <TopBar {inboxes} {currentInboxId} onselect={select} onmanage={() => (showInboxes = true)} onoptions={() => (showOptions = true)} />
  <div class="grid min-h-0 flex-1 grid-cols-[calc(var(--spacing)*90)_1fr] gap-3">
    <MailList inboxId={currentInboxId} selectedId={selectedMailId} refreshKey={listKey} onselect={(id) => (selectedMailId = id)} onchanged={listChanged} />
    <MailView
      mailId={selectedMailId}
      onread={() => reload()}
      ondeleted={() => {
        selectedMailId = null;
        refresh();
      }}
    />
  </div>
</div>

{#if showInboxes}
  <InboxesDialog {inboxes} onchanged={inboxesChanged} onclose={() => (showInboxes = false)} />
{/if}

{#if showOptions}
  <OptionsDialog onclose={() => (showOptions = false)} />
{/if}
