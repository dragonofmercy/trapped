<script lang="ts">
  import { untrack } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { save } from "@tauri-apps/plugin-dialog";
  import { deleteMails, getMail, getRaw, openAttachment, saveAttachment, type MailDetail } from "../api";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Icon from "./Icon.svelte";

  let { mailId, onread, ondeleted }: { mailId: number | null; onread: () => void; ondeleted: () => void } = $props();

  type Tab = "html" | "text" | "source" | "headers" | "attachments";
  let mail = $state<MailDetail | null>(null);
  let tab = $state<Tab>("text");
  let mobile = $state(false);
  let error = $state("");
  let actionError = $state("");
  let confirmDelete = $state(false);

  // Only the latest request may write the mail (fast selection changes)
  let seq = 0;
  $effect(() => {
    const id = mailId;
    untrack(() => load(id));
  });
  async function load(id: number | null) {
    const n = ++seq;
    actionError = "";
    if (id === null) {
      mail = null;
      error = "";
      return;
    }
    try {
      const m = await getMail(id);
      if (n !== seq) return;
      mail = m;
      error = "";
      tab = m.html ? "html" : "text";
      onread();
    } catch (e) {
      if (n !== seq) return;
      mail = null;
      error = String(e);
    }
  }

  // Raw source is loaded lazily when the Source tab opens (cached per mail id)
  let raw = $state<{ id: number; text: string } | null>(null);
  let rawError = $state("");
  let rawSeq = 0;
  $effect(() => {
    const id = mail?.id;
    if (tab !== "source" || id === undefined) return;
    untrack(async () => {
      if (raw?.id === id) return;
      const n = ++rawSeq;
      rawError = "";
      try {
        const text = await getRaw(id);
        if (n === rawSeq) raw = { id, text };
      } catch (e) {
        if (n === rawSeq) rawError = String(e);
      }
    });
  });

  const tabs = $derived<{ id: Tab; label: string }[]>([
    { id: "html", label: "HTML" },
    { id: "text", label: "Text" },
    { id: "source", label: "Source" },
    { id: "headers", label: "Headers" },
    { id: "attachments", label: `Attachments (${mail?.attachments.length ?? 0})` },
  ]);

  const dateText = $derived.by(() => {
    if (!mail) return "";
    const d = mail.date ? new Date(mail.date) : new Date(mail.receivedAt * 1000);
    return isNaN(d.getTime()) ? (mail.date ?? "") : d.toLocaleString();
  });

  function fmtSize(b: number) {
    return b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`;
  }

  // The sandboxed iframe cannot run mail scripts; the parent handles links and blocks navigation
  function onFrameLoad(ev: Event) {
    const doc = (ev.target as HTMLIFrameElement).contentDocument;
    if (!doc) return;
    doc.addEventListener("click", (e) => {
      const a = (e.target as Element | null)?.closest?.("a[href], area[href]");
      if (!a) return;
      // in-page anchors keep working
      if (a.getAttribute("href")?.trim().startsWith("#")) return;
      e.preventDefault();
      const url = (a as HTMLAnchorElement | HTMLAreaElement).href;
      if (/^(https?:|mailto:)/i.test(url)) openUrl(url).catch((err) => (actionError = String(err)));
    });
    doc.addEventListener("submit", (e) => e.preventDefault());
  }

  async function run(fn: () => Promise<unknown>) {
    actionError = "";
    try {
      await fn();
    } catch (e) {
      actionError = String(e);
    }
  }

  const saveAtt = (index: number, name: string) =>
    run(async () => {
      const path = await save({ defaultPath: name });
      if (path && mail) await saveAttachment(mail.id, index, path);
    });

  async function remove() {
    confirmDelete = false;
    const id = mail?.id;
    if (id === undefined) return;
    actionError = "";
    try {
      await deleteMails([id]);
      ondeleted();
    } catch (e) {
      actionError = String(e);
    }
  }
</script>

<div class="card flex min-h-0 flex-col gap-3 p-3">
  {#if error}
    <p class="selectable m-auto text-danger">{error}</p>
  {:else if !mail}
    <p class="m-auto text-muted">Select a mail</p>
  {:else}
    {@const m = mail}
    <div class="flex items-start gap-3">
      <div class="selectable min-w-0 flex-1">
        <h2 class="m-0 text-base font-bold break-words text-text">{m.subject || "(no subject)"}</h2>
        <div class="text-muted"><span class="label">From</span> {m.from}</div>
        {#if m.to.length}<div class="text-muted"><span class="label">To</span> {m.to.join(", ")}</div>{/if}
        {#if m.cc.length}<div class="text-muted"><span class="label">Cc</span> {m.cc.join(", ")}</div>{/if}
        {#if m.bcc.length}<div class="text-muted"><span class="label">Bcc</span> {m.bcc.join(", ")}</div>{/if}
        <div class="text-dim">{dateText}</div>
      </div>
      <button type="button" class="btn danger shrink-0" onclick={() => (confirmDelete = true)}><Icon name="trash" />Delete</button>
    </div>
    {#if actionError}<div class="selectable text-danger">{actionError}</div>{/if}

    <div class="flex items-center gap-1 border-b border-line" role="tablist">
      {#each tabs as t (t.id)}
        <button
          type="button"
          role="tab"
          aria-selected={tab === t.id}
          class={["-mb-px cursor-pointer border-0 border-b-2 bg-transparent px-3 py-2 text-inherit", tab === t.id ? "border-accent text-text" : "border-transparent text-muted hover:text-text"]}
          onclick={() => (tab = t.id)}
        >
          {t.label}
        </button>
      {/each}
      {#if tab === "html"}
        <div class="seg ml-auto" role="group" aria-label="Preview width">
          <button type="button" aria-pressed={!mobile} onclick={() => (mobile = false)}><Icon name="monitor" />Desktop</button>
          <button type="button" aria-pressed={mobile} onclick={() => (mobile = true)}><Icon name="phone" />Mobile</button>
        </div>
      {/if}
    </div>

    <div class="min-h-0 flex-1 overflow-auto">
      {#if tab === "html"}
        {#if !m.html}
          <p class="m-0 p-4 text-center text-muted">This mail has no HTML part.</p>
        {:else}
          <iframe sandbox="allow-same-origin" srcdoc={m.html} title="Mail HTML" class="mx-auto block h-full border-0 bg-white" style:width={mobile ? "390px" : "100%"} onload={onFrameLoad}></iframe>
        {/if}
      {:else if tab === "text"}
        <pre class="selectable m-0 font-mono break-words whitespace-pre-wrap">{m.text ?? "(no text part)"}</pre>
      {:else if tab === "source"}
        {#if rawError}
          <p class="selectable m-0 text-danger">{rawError}</p>
        {:else}
          <pre class="selectable m-0 font-mono">{raw?.id === m.id ? raw.text : "Loading..."}</pre>
        {/if}
      {:else if tab === "headers"}
        <table class="selectable w-full border-collapse font-mono">
          <tbody>
            {#each m.headers as h, i (i)}
              <tr class="border-b border-line align-top">
                <th class="py-1 pr-4 text-left font-bold whitespace-nowrap text-muted">{h[0]}</th>
                <td class="py-1 break-all">{h[1]}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        {#if !m.attachments.length}
          <p class="m-0 p-4 text-center text-muted">No attachments.</p>
        {/if}
        {#each m.attachments as a (a.index)}
          <div class="flex items-center gap-3 border-b border-line py-2">
            <div class="min-w-0 flex-1">
              <div class="selectable truncate">{a.name}</div>
              <div class="text-muted">{a.mime} - {fmtSize(a.size)}</div>
            </div>
            <button
              type="button"
              class="btn"
              disabled={a.blocked}
              title={a.blocked ? "Executable file types cannot be opened. Save it instead." : undefined}
              onclick={() => run(() => openAttachment(m.id, a.index))}
            >
              <Icon name="external" />Open
            </button>
            <button type="button" class="btn" onclick={() => saveAtt(a.index, a.name)}><Icon name="download" />Save</button>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

{#if confirmDelete}
  <ConfirmDialog title="Delete Mail" message="Delete this mail?" confirmLabel="Delete" onconfirm={remove} oncancel={() => (confirmDelete = false)} />
{/if}
