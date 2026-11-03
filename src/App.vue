<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { listInboxes, onMailReceived, type Inbox } from "./api";
import TopBar from "./components/TopBar.vue";

const inboxes = ref<Inbox[]>([]);
const currentInboxId = ref(0);
const selectedMailId = ref<number | null>(null);

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
  await reload();
  unlisten = await onMailReceived(() => reload());
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <div class="flex h-full flex-col gap-3 p-3">
    <TopBar :inboxes="inboxes" :current-inbox-id="currentInboxId" @select="select" @changed="reload" />
    <div class="grid min-h-0 flex-1 grid-cols-[360px_1fr] gap-3">
      <!-- Task 7: <MailList :inbox-id="currentInboxId" :selected-id="selectedMailId" @select="selectedMailId = $event" @changed="reload()" /> -->
      <div class="card"></div>
      <!-- Task 8: <MailView :mail-id="selectedMailId" @deleted="selectedMailId = null; reload()" /> -->
      <div class="card"></div>
    </div>
  </div>
</template>
