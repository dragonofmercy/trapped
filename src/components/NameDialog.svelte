<script lang="ts">
  import Modal from "./Modal.svelte";

  // validate returns null when the name is valid, "" when invalid without a message worth showing (empty field)
  let { title, initial, validate, error = "", onconfirm, oncancel }: { title: string; initial: string; validate: (name: string) => string | null; error?: string; onconfirm: (name: string) => void; oncancel: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let name = $state(initial);
  let invalid = $derived(validate(name));

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (invalid === null) onconfirm(name.trim());
  }
</script>

<Modal {title} width={320} onclose={oncancel}>
  <form onsubmit={submit}>
    <!-- autofocus wins over the close cross when the modal opens; the current name is selected for overtyping -->
    <!-- svelte-ignore a11y_autofocus -->
    <input class="field w-full" bind:value={name} autofocus onfocus={(e) => e.currentTarget.select()} />
    <p class="error selectable mt-1.5 min-h-[1.2em]">{invalid || error}</p>
    <div class="buttons mt-1.5">
      <button type="button" class="btn ghost" onclick={oncancel}>Cancel</button>
      <button type="submit" class="btn" disabled={invalid !== null}>OK</button>
    </div>
  </form>
</Modal>
