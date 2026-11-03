<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { listInboxes, onMailReceived, type Inbox } from "./api";
import MailList from "./components/MailList.vue";
import MailView from "./components/MailView.vue";
import TopBar from "./components/TopBar.vue";

const inboxes = ref<Inbox[]>([]);
const currentInboxId = ref(0);
const selectedMailId = ref<number | null>(null);
const listKey = ref(0);

function select(id: number) {
  currentInboxId.value = id;
  selectedMailId.value = null;
}

async function reload(selectId?: number) {
  inboxes.value = await listInboxes();
  if (selectId !== undefined) select(selectId);
  else if (!inboxes.value.some((i) => i.id === currentInboxId.value)) select(inboxes.value[0].id);
}

let unlisten: (() => void) | undefined;
onMounted(async () => {
  unlisten = await onMailReceived(() => reload().catch(() => {}));
  await reload().catch(() => {});
});
onBeforeUnmount(() => unlisten?.());

function refresh() {
  listKey.value++;
  reload().catch(() => {});
}

function topChanged(selectId?: number) {
  listKey.value++;
  selectedMailId.value = null;
  reload(selectId).catch(() => {});
}

function listChanged(deletedIds: number[]) {
  if (selectedMailId.value !== null && deletedIds.includes(selectedMailId.value)) selectedMailId.value = null;
  reload().catch(() => {});
}
</script>

<template>
  <div class="flex h-full flex-col gap-3 p-3">
    <TopBar :inboxes="inboxes" :current-inbox-id="currentInboxId" @select="select" @changed="topChanged" />
    <div class="grid min-h-0 flex-1 grid-cols-[360px_1fr] gap-3">
      <MailList :inbox-id="currentInboxId" :selected-id="selectedMailId" :refresh-key="listKey" @select="selectedMailId = $event" @changed="listChanged" />
      <MailView :mail-id="selectedMailId" @read="refresh" @deleted="selectedMailId = null; refresh()" />
    </div>
  </div>
</template>
