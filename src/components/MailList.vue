<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { deleteMails, listMails, onMailReceived, type MailRow } from "../api";

const props = defineProps<{ inboxId: number; selectedId: number | null; refreshKey: number }>();
const emit = defineEmits<{ select: [id: number]; changed: [deletedIds: number[]] }>();

const rows = ref<MailRow[]>([]);
const checked = ref<number[]>([]);
const query = ref("");
const error = ref("");
const deleteError = ref("");

// Only the latest request may write rows (search typing and events can overlap)
let seq = 0;
async function load() {
  const n = ++seq;
  try {
    const r = await listMails(props.inboxId, query.value);
    if (n !== seq) return;
    rows.value = r;
    checked.value = checked.value.filter((id) => r.some((m) => m.id === id));
    error.value = "";
  } catch (e) {
    if (n === seq) error.value = String(e);
  }
}

let timer: ReturnType<typeof setTimeout> | undefined;
watch(query, () => {
  clearTimeout(timer);
  timer = setTimeout(load, 200);
});
watch(
  () => props.inboxId,
  () => {
    checked.value = [];
    deleteError.value = "";
    clearTimeout(timer);
    load();
  },
);
watch(() => props.refreshKey, load);

let unlisten: (() => void) | undefined;
let unmounted = false;
onMounted(async () => {
  load();
  const un = await onMailReceived((id) => {
    if (id === props.inboxId) load();
  });
  if (unmounted) un();
  else unlisten = un;
});
onBeforeUnmount(() => {
  unmounted = true;
  unlisten?.();
  clearTimeout(timer);
});

const allChecked = computed(() => rows.value.length > 0 && checked.value.length === rows.value.length);
const someChecked = computed(() => checked.value.length > 0 && !allChecked.value);
function toggleAll() {
  checked.value = allChecked.value ? [] : rows.value.map((m) => m.id);
}

function fmtTime(secs: number) {
  const d = new Date(secs * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  if (d.toDateString() === new Date().toDateString()) return `${p(d.getHours())}:${p(d.getMinutes())}`;
  return `${p(d.getDate())}.${p(d.getMonth() + 1)}.${d.getFullYear()}`;
}

// Confirm dialog
const dlg = ref<HTMLDialogElement>();
async function confirmDelete() {
  const ids = [...checked.value];
  dlg.value?.close();
  deleteError.value = "";
  let ok = false;
  try {
    await deleteMails(ids);
    checked.value = [];
    ok = true;
  } catch (e) {
    deleteError.value = String(e);
  }
  await load();
  if (ok) emit("changed", ids);
}
</script>

<template>
  <div class="card flex min-h-0 flex-col gap-2 p-3">
    <input v-model="query" class="field w-full" type="search" placeholder="Search mails" />

    <div class="flex h-8 items-center gap-2 px-1">
      <input type="checkbox" :checked="allChecked" :indeterminate="someChecked" :disabled="!rows.length" aria-label="Select all" @change="toggleAll" />
      <span class="label">Mails</span>
      <button v-if="checked.length" class="btn danger ml-auto h-7" @click="dlg?.showModal()">Delete selection ({{ checked.length }})</button>
    </div>
    <div v-if="deleteError || error" class="selectable text-danger">{{ deleteError || error }}</div>

    <div class="min-h-0 flex-1 overflow-y-auto">
      <p v-if="!rows.length" class="m-0 p-4 text-center text-muted">{{ query ? "No mail matches." : "No mail yet. Send to 127.0.0.1:2525." }}</p>
      <div
        v-for="m in rows"
        :key="m.id"
        role="button"
        tabindex="0"
        :aria-label="m.subject || '(no subject)'"
        :aria-current="m.id === selectedId"
        class="flex cursor-pointer items-center gap-2 border-l-2 px-2 py-2 hover:bg-bg2/40"
        :class="m.id === selectedId ? 'border-accent bg-bg2/60' : 'border-transparent'"
        @click="emit('select', m.id)"
        @keydown.enter.self.prevent="emit('select', m.id)"
        @keydown.space.self.prevent="emit('select', m.id)"
      >
        <input v-model="checked" type="checkbox" :value="m.id" aria-label="Select this mail" @click.stop />
        <div class="min-w-0 flex-1">
          <div class="truncate" :class="m.isRead ? 'text-muted' : 'font-bold text-text'">{{ m.subject || "(no subject)" }}</div>
          <div class="truncate text-muted">{{ m.from }}</div>
        </div>
        <svg v-if="m.hasAttachments" class="size-4 shrink-0 text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
        </svg>
        <span class="shrink-0 font-mono text-muted">{{ fmtTime(m.receivedAt) }}</span>
      </div>
    </div>

    <dialog ref="dlg">
      <p class="m-0">Delete {{ checked.length }} selected {{ checked.length === 1 ? "mail" : "mails" }}?</p>
      <div class="buttons">
        <button class="btn ghost" @click="dlg?.close()">Cancel</button>
        <button class="btn danger" @click="confirmDelete">Delete</button>
      </div>
    </dialog>
  </div>
</template>
