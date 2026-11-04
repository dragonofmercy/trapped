<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  // Shared modal frame: header (title + close cross) above a padded body; width in Tailwind spacing units
  let { title, width = 90, onclose, children }: { title: string; width?: number; onclose: () => void; children: Snippet } = $props();

  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());
</script>

<dialog bind:this={dialog} style:width="calc(var(--spacing) * {width})" oncancel={(e) => (e.preventDefault(), onclose())}>
  <header>
    <h2>{title}</h2>
    <button type="button" class="btn ghost close" title="Close" aria-label="Close" onclick={onclose}><Icon name="close" /></button>
  </header>
  <div class="body">{@render children()}</div>
</dialog>

<style>
  dialog {
    max-width: 90vw;
    padding: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 2);
    padding: calc(var(--spacing) * 2.5) calc(var(--spacing) * 2.5) calc(var(--spacing) * 2.5) calc(var(--spacing) * 4.5);
    border-bottom: 1px solid var(--line);
  }
  h2 {
    flex: 1;
    margin: 0;
  }
  .close {
    width: calc(var(--spacing) * 8.5);
    padding: 0;
    justify-content: center;
    border-color: transparent;
  }
  .body {
    padding: calc(var(--spacing) * 4.5);
  }
</style>
