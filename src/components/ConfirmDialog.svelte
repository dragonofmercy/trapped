<script lang="ts">
  import Modal from "./Modal.svelte";

  let { title, message, confirmLabel, onconfirm, oncancel }: { title: string; message: string; confirmLabel: string; onconfirm: () => void; oncancel: () => void } = $props();
</script>

<Modal {title} class="w-110" onclose={oncancel}>
  <div class="space-y-4">
    <div class="space-y-1">
      <p>{message}</p>
      <p class="text-muted">This action cannot be undone.</p>
    </div>
    <div class="buttons">
      <!-- Focus on Cancel: a reflex Enter must not confirm. -->
      <!-- svelte-ignore a11y_autofocus -->
      <button type="button" class="btn ghost" onclick={oncancel} autofocus>Cancel</button>
      <button type="button" class="btn danger" onclick={onconfirm}>{confirmLabel}</button>
    </div>
  </div>
</Modal>
