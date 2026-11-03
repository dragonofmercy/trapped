<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { emptyInbox, smtpRetry, smtpStatus, onSmtpStatus, type Inbox, type SmtpStatus } from "../api";

const props = defineProps<{ inboxes: Inbox[]; currentInboxId: number }>();
const emit = defineEmits<{ select: [id: number]; changed: [selectId?: number]; options: [] }>();

const current = computed(() => props.inboxes.find((i) => i.id === props.currentInboxId));

const status = ref<SmtpStatus | null>(null);
let unlisten: (() => void) | undefined;
onMounted(async () => {
  unlisten = await onSmtpStatus((s) => (status.value = s));
  try {
    status.value = await smtpStatus();
  } catch (e) {
    status.value = { listening: false, address: "", error: String(e) };
  }
});
onBeforeUnmount(() => unlisten?.());

const retrying = ref(false);
async function retry() {
  retrying.value = true;
  try {
    status.value = await smtpRetry();
  } catch (e) {
    status.value = { listening: false, address: "", error: String(e) };
  } finally {
    retrying.value = false;
  }
}

const actionError = ref("");
watch(
  () => props.currentInboxId,
  () => (actionError.value = ""),
);

// Confirm dialog for Empty
const dlg = ref<HTMLDialogElement>();
async function confirm() {
  dlg.value?.close();
  actionError.value = "";
  try {
    await emptyInbox(props.currentInboxId);
    emit("changed", props.currentInboxId);
  } catch (e) {
    actionError.value = String(e);
    emit("changed");
  }
}
</script>

<template>
  <div class="card flex items-center gap-3 py-2.5">
    <span class="label">Inbox</span>
    <select class="field" :value="currentInboxId" @change="emit('select', Number(($event.target as HTMLSelectElement).value))">
      <option v-for="i in inboxes" :key="i.id" :value="i.id">{{ i.name }} ({{ i.count }})</option>
    </select>

    <button class="btn warn" :disabled="!current?.count" @click="dlg?.showModal()">Empty</button>
    <span v-if="actionError" class="selectable text-danger">{{ actionError }}</span>

    <div class="ml-auto flex items-center gap-2">
      <template v-if="status?.listening">
        <span class="size-2 rounded-full bg-ok"></span>
        <span class="font-mono text-muted">{{ status.address }}</span>
      </template>
      <template v-else-if="status">
        <span class="size-2 rounded-full bg-danger"></span>
        <span class="selectable text-danger">{{ status.error ?? "SMTP server not running" }}</span>
        <button class="btn" :disabled="retrying" @click="retry">Retry</button>
      </template>
      <button class="btn ghost" title="Options" @click="emit('options')">Options</button>
    </div>

    <dialog ref="dlg">
      <p class="m-0">Delete all mails in "{{ current?.name }}"?</p>
      <div class="buttons">
        <button class="btn ghost" @click="dlg?.close()">Cancel</button>
        <button class="btn danger" @click="confirm">Empty</button>
      </div>
    </dialog>
  </div>
</template>
