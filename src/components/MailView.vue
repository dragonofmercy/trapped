<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { save } from "@tauri-apps/plugin-dialog";
import { deleteMails, getMail, openAttachment, saveAttachment, type MailDetail } from "../api";

const props = defineProps<{ mailId: number | null }>();
const emit = defineEmits<{ deleted: []; read: [] }>();

type Tab = "html" | "text" | "source" | "headers" | "attachments";
const mail = ref<MailDetail | null>(null);
const tab = ref<Tab>("text");
const mobile = ref(false);
const error = ref("");
const actionError = ref("");

// Only the latest request may write the mail (fast selection changes)
let seq = 0;
watch(
  () => props.mailId,
  async (id) => {
    const n = ++seq;
    actionError.value = "";
    if (id === null) {
      mail.value = null;
      error.value = "";
      return;
    }
    try {
      const m = await getMail(id);
      if (n !== seq) return;
      mail.value = m;
      error.value = "";
      tab.value = m.html ? "html" : "text";
      emit("read");
    } catch (e) {
      if (n !== seq) return;
      mail.value = null;
      error.value = String(e);
    }
  },
  { immediate: true },
);

const tabs = computed<{ id: Tab; label: string }[]>(() => [
  { id: "html", label: "HTML" },
  { id: "text", label: "Text" },
  { id: "source", label: "Source" },
  { id: "headers", label: "Headers" },
  { id: "attachments", label: `Attachments (${mail.value?.attachments.length ?? 0})` },
]);

const dateText = computed(() => {
  const m = mail.value;
  if (!m) return "";
  const d = m.date ? new Date(m.date) : new Date(m.receivedAt * 1000);
  return isNaN(d.getTime()) ? (m.date ?? "") : d.toLocaleString();
});

function fmtSize(b: number) {
  return b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`;
}

// The sandboxed iframe cannot run mail scripts; the parent handles links and blocks navigation
function onFrameLoad(ev: Event) {
  const doc = (ev.target as HTMLIFrameElement).contentDocument;
  if (!doc) return;
  doc.addEventListener("click", (e) => {
    const a = (e.target as Element | null)?.closest?.("a[href]");
    if (!a) return;
    e.preventDefault();
    const href = a.getAttribute("href") ?? "";
    if (/^(https?:|mailto:)/i.test(href)) openUrl(href).catch((err) => (actionError.value = String(err)));
  });
  doc.addEventListener("submit", (e) => e.preventDefault());
}

async function run(fn: () => Promise<unknown>) {
  actionError.value = "";
  try {
    await fn();
  } catch (e) {
    actionError.value = String(e);
  }
}

const saveAtt = (index: number, name: string) =>
  run(async () => {
    const path = await save({ defaultPath: name });
    if (path && mail.value) await saveAttachment(mail.value.id, index, path);
  });

// Confirm dialog
const dlg = ref<HTMLDialogElement>();
async function confirmDelete() {
  dlg.value?.close();
  const id = mail.value?.id;
  if (id === undefined) return;
  actionError.value = "";
  try {
    await deleteMails([id]);
    emit("deleted");
  } catch (e) {
    actionError.value = String(e);
  }
}
</script>

<template>
  <div class="card flex min-h-0 flex-col gap-3 p-3">
    <p v-if="error" class="selectable m-auto text-danger">{{ error }}</p>
    <p v-else-if="!mail" class="m-auto text-muted">Select a mail</p>
    <template v-else>
      <div class="flex items-start gap-3">
        <div class="selectable min-w-0 flex-1">
          <h2 class="m-0 break-words text-base font-bold text-text">{{ mail.subject || "(no subject)" }}</h2>
          <div class="text-muted"><span class="label">From</span> {{ mail.from }}</div>
          <div v-if="mail.to.length" class="text-muted"><span class="label">To</span> {{ mail.to.join(", ") }}</div>
          <div v-if="mail.cc.length" class="text-muted"><span class="label">Cc</span> {{ mail.cc.join(", ") }}</div>
          <div v-if="mail.bcc.length" class="text-muted"><span class="label">Bcc</span> {{ mail.bcc.join(", ") }}</div>
          <div class="text-dim">{{ dateText }}</div>
        </div>
        <button class="btn danger shrink-0" @click="dlg?.showModal()">Delete</button>
      </div>
      <div v-if="actionError" class="selectable text-danger">{{ actionError }}</div>

      <div class="flex items-center gap-1 border-b border-line" role="tablist">
        <button
          v-for="t in tabs"
          :key="t.id"
          role="tab"
          :aria-selected="tab === t.id"
          class="-mb-px cursor-pointer border-0 border-b-2 bg-transparent px-3 py-2 text-inherit"
          :class="tab === t.id ? 'border-accent text-text' : 'border-transparent text-muted hover:text-text'"
          @click="tab = t.id"
        >
          {{ t.label }}
        </button>
        <div v-if="tab === 'html'" class="ml-auto flex gap-1">
          <button class="btn h-7" :class="mobile ? 'ghost' : 'primary'" :aria-pressed="!mobile" @click="mobile = false">Desktop</button>
          <button class="btn h-7" :class="mobile ? 'primary' : 'ghost'" :aria-pressed="mobile" @click="mobile = true">Mobile</button>
        </div>
      </div>

      <div class="min-h-0 flex-1 overflow-auto">
        <template v-if="tab === 'html'">
          <p v-if="!mail.html" class="m-0 p-4 text-center text-muted">This mail has no HTML part.</p>
          <iframe
            v-else
            sandbox="allow-same-origin"
            :srcdoc="mail.html"
            title="Mail HTML"
            class="mx-auto block h-full border-0 bg-white"
            :style="{ width: mobile ? '390px' : '100%' }"
            @load="onFrameLoad"
          ></iframe>
        </template>
        <pre v-else-if="tab === 'text'" class="selectable m-0 whitespace-pre-wrap break-words font-mono">{{ mail.text ?? "(no text part)" }}</pre>
        <pre v-else-if="tab === 'source'" class="selectable m-0 font-mono">{{ mail.raw }}</pre>
        <table v-else-if="tab === 'headers'" class="selectable w-full border-collapse font-mono">
          <tr v-for="(h, i) in mail.headers" :key="i" class="border-b border-line align-top">
            <th class="whitespace-nowrap py-1 pr-4 text-left font-bold text-muted">{{ h[0] }}</th>
            <td class="break-all py-1">{{ h[1] }}</td>
          </tr>
        </table>
        <div v-else>
          <p v-if="!mail.attachments.length" class="m-0 p-4 text-center text-muted">No attachments.</p>
          <div v-for="a in mail.attachments" :key="a.index" class="flex items-center gap-3 border-b border-line py-2">
            <div class="min-w-0 flex-1">
              <div class="selectable truncate">{{ a.name }}</div>
              <div class="text-muted">{{ a.mime }} - {{ fmtSize(a.size) }}</div>
            </div>
            <button
              class="btn"
              :disabled="a.blocked"
              :title="a.blocked ? 'Executable file types cannot be opened. Save it instead.' : undefined"
              @click="run(() => openAttachment(mail!.id, a.index))"
            >
              Open
            </button>
            <button class="btn" @click="saveAtt(a.index, a.name)">Save</button>
          </div>
        </div>
      </div>
    </template>

    <dialog ref="dlg">
      <p class="m-0">Delete this mail?</p>
      <div class="buttons">
        <button class="btn ghost" @click="dlg?.close()">Cancel</button>
        <button class="btn danger" @click="confirmDelete">Delete</button>
      </div>
    </dialog>
  </div>
</template>
