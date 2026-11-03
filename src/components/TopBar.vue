<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { deleteInbox, emptyInbox, renameInbox, smtpRetry, smtpStatus, onSmtpStatus, type Inbox, type SmtpStatus } from "../api";

const props = defineProps<{ inboxes: Inbox[]; currentInboxId: number }>();
const emit = defineEmits<{ select: [id: number]; changed: [selectId?: number] }>();

const current = computed(() => props.inboxes.find((i) => i.id === props.currentInboxId));
const isDefault = computed(() => current.value?.name === "Default");

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

// Rename (inline input)
const renaming = ref(false);
const newName = ref("");
const renameError = ref("");
const renameInput = ref<HTMLInputElement>();
const actionError = ref("");
// A pending rename or error belongs to the inbox it was started on
watch(
  () => props.currentInboxId,
  () => {
    renaming.value = false;
    renameError.value = "";
    actionError.value = "";
  },
);
function cancelRename() {
  renaming.value = false;
  renameError.value = "";
}
async function startRename() {
  newName.value = current.value?.name ?? "";
  renameError.value = "";
  actionError.value = "";
  renaming.value = true;
  await nextTick();
  renameInput.value?.select();
}
async function confirmRename() {
  try {
    await renameInbox(props.currentInboxId, newName.value);
    renaming.value = false;
    emit("changed");
  } catch (e) {
    renameError.value = String(e);
  }
}

// Confirm dialog for Empty / Delete
const dlg = ref<HTMLDialogElement>();
const pending = ref<"empty" | "delete" | null>(null);
function ask(kind: "empty" | "delete") {
  pending.value = kind;
  dlg.value?.showModal();
}
async function confirm() {
  const kind = pending.value;
  dlg.value?.close();
  actionError.value = "";
  try {
    if (kind === "empty") {
      await emptyInbox(props.currentInboxId);
      emit("changed", props.currentInboxId);
    } else if (kind === "delete") {
      await deleteInbox(props.currentInboxId);
      emit("changed", props.inboxes.find((i) => i.name === "Default")?.id);
    }
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

    <template v-if="renaming">
      <input ref="renameInput" v-model="newName" class="field" @input="renameError = ''" @keydown.enter="confirmRename" @keydown.esc="cancelRename" />
      <span v-if="renameError" class="selectable text-danger">{{ renameError }}</span>
    </template>
    <template v-else>
      <button class="btn" :disabled="isDefault" @click="startRename">Rename</button>
      <button class="btn warn" :disabled="!current?.count" @click="ask('empty')">Empty</button>
      <button v-if="!isDefault" class="btn danger" @click="ask('delete')">Delete</button>
      <span v-if="actionError" class="selectable text-danger">{{ actionError }}</span>
    </template>

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
    </div>

    <dialog ref="dlg">
      <p class="m-0">
        <template v-if="pending === 'empty'">Delete all mails in "{{ current?.name }}"?</template>
        <template v-else>Delete inbox "{{ current?.name }}" and all its mails?</template>
      </p>
      <div class="buttons">
        <button class="btn ghost" @click="dlg?.close()">Cancel</button>
        <button class="btn danger" @click="confirm">{{ pending === "empty" ? "Empty" : "Delete" }}</button>
      </div>
    </dialog>
  </div>
</template>
