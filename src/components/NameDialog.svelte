<script lang="ts">
  import { onMount } from "svelte";

  // validate returns null when the name is valid, "" when invalid without a message worth showing (empty field)
  let { title, initial, validate, error = "", onconfirm, oncancel }: { title: string; initial: string; validate: (name: string) => string | null; error?: string; onconfirm: (name: string) => void; oncancel: () => void } = $props();

  let dialog: HTMLDialogElement;
  let input: HTMLInputElement;
  // svelte-ignore state_referenced_locally
  let name = $state(initial);
  let invalid = $derived(validate(name));

  onMount(() => {
    dialog.showModal();
    input.select();
  });

  function submit(e: SubmitEvent) {
    e.preventDefault();
    if (invalid === null) onconfirm(name.trim());
  }
</script>

<dialog bind:this={dialog} oncancel={(e) => (e.preventDefault(), oncancel())}>
  <form onsubmit={submit}>
    <h2>{title}</h2>
    <input class="field" bind:this={input} bind:value={name} />
    <p class="error selectable">{invalid || error}</p>
    <div class="buttons">
      <button type="button" class="btn ghost" onclick={oncancel}>Cancel</button>
      <button type="submit" class="btn primary" disabled={invalid !== null}>OK</button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    width: 320px;
  }
  input {
    width: 100%;
  }
  .error {
    min-height: 1.2em;
    margin: 6px 0 0;
  }
  .buttons {
    margin-top: 6px;
  }
</style>
