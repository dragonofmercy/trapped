<script setup lang="ts">
import { nextTick, ref } from "vue";
import { createInbox, deleteInbox, renameInbox, type Inbox } from "../api";
import { theme, type Theme } from "../theme";

defineProps<{ inboxes: Inbox[] }>();
const emit = defineEmits<{ changed: [selectId?: number] }>();

const dlg = ref<HTMLDialogElement>();
const section = ref<"general" | "inboxes">("general");
const themes: [Theme, string][] = [
  ["system", "System"],
  ["dark", "Dark"],
  ["light", "Light"],
];

// One inline action at a time: renaming or confirming a delete on a single row
const editId = ref<number | null>(null);
const editName = ref("");
const deleteId = ref<number | null>(null);
const newName = ref("");
const error = ref("");
const editInput = ref<HTMLInputElement[]>();

function reset() {
  editId.value = null;
  deleteId.value = null;
  error.value = "";
}

function open() {
  reset();
  newName.value = "";
  dlg.value?.showModal();
}
defineExpose({ open });

async function run(action: () => Promise<unknown>, selectId?: number) {
  error.value = "";
  try {
    await action();
    reset();
    emit("changed", selectId);
  } catch (e) {
    error.value = String(e);
  }
}

async function add() {
  error.value = "";
  try {
    const id = await createInbox(newName.value);
    newName.value = "";
    reset();
    emit("changed", id);
  } catch (e) {
    error.value = String(e);
  }
}

async function startRename(i: Inbox) {
  reset();
  editId.value = i.id;
  editName.value = i.name;
  await nextTick();
  editInput.value?.[0]?.select();
}

function confirmRename(id: number) {
  run(() => renameInbox(id, editName.value));
}

function confirmDelete(id: number) {
  run(() => deleteInbox(id));
}

// Esc inside an inline edit cancels the edit, not the whole dialog
function onCancel(e: Event) {
  if (editId.value !== null || deleteId.value !== null) {
    e.preventDefault();
    reset();
  }
}
</script>

<template>
  <dialog ref="dlg" class="w-[720px] max-w-[90vw] p-0" @cancel="onCancel">
    <div class="flex h-[440px]">
      <nav class="flex w-44 flex-col gap-1 border-r border-line p-3">
        <span class="label mb-2 px-2">Options</span>
        <button class="nav" :class="{ on: section === 'general' }" @click="section = 'general'; reset()">General</button>
        <button class="nav" :class="{ on: section === 'inboxes' }" @click="section = 'inboxes'; reset()">Inboxes</button>
      </nav>

      <section class="flex min-w-0 flex-1 flex-col p-5">
        <template v-if="section === 'general'">
          <span class="label">Theme</span>
          <div class="mt-3 flex gap-2">
            <button v-for="[value, text] in themes" :key="value" class="btn" :class="{ primary: theme === value }" @click="theme = value">{{ text }}</button>
          </div>
        </template>

        <template v-else>
          <span class="label">Inboxes</span>
          <form class="mt-3 flex gap-2" @submit.prevent="add">
            <input v-model="newName" class="field flex-1" placeholder="New inbox name" @input="error = ''" />
            <button class="btn primary" :disabled="!newName.trim()">Add</button>
          </form>
          <p v-if="error" class="selectable m-0 mt-2 text-danger">{{ error }}</p>

          <ul class="m-0 mt-3 flex min-h-0 flex-1 list-none flex-col gap-1.5 overflow-y-auto p-0">
            <li v-for="i in inboxes" :key="i.id" class="flex h-11 items-center gap-2 rounded-lg border border-line px-3">
              <template v-if="editId === i.id">
                <input ref="editInput" v-model="editName" class="field h-8 flex-1" @input="error = ''" @keydown.enter="confirmRename(i.id)" />
                <button class="btn h-8" @click="reset">Cancel</button>
                <button class="btn primary h-8" @click="confirmRename(i.id)">Save</button>
              </template>
              <template v-else-if="deleteId === i.id">
                <span class="min-w-0 flex-1 truncate">Delete "{{ i.name }}" and its {{ i.count }} mail(s)?</span>
                <button class="btn h-8" @click="reset">Cancel</button>
                <button class="btn danger h-8" @click="confirmDelete(i.id)">Delete</button>
              </template>
              <template v-else>
                <span class="min-w-0 flex-1 truncate font-medium">{{ i.name }}</span>
                <span class="font-mono text-muted">{{ i.count }}</span>
                <template v-if="i.name !== 'Default'">
                  <button class="btn ghost h-8" @click="startRename(i)">Rename</button>
                  <button class="btn ghost h-8" @click="reset(); deleteId = i.id">Delete</button>
                </template>
              </template>
            </li>
          </ul>
        </template>

        <div class="buttons">
          <button class="btn" @click="dlg?.close()">Close</button>
        </div>
      </section>
    </div>
  </dialog>
</template>

<style scoped>
.nav {
  text-align: left;
  padding: 8px 10px;
  border-radius: 8px;
  border: 0;
  background: transparent;
  color: var(--muted);
  font: 500 13px Inter, sans-serif;
  cursor: pointer;
}

.nav:hover {
  background: rgba(127, 127, 127, 0.1);
  color: var(--text);
}

.nav.on {
  background: rgba(var(--accent-rgb), 0.18);
  color: var(--text);
}
</style>
