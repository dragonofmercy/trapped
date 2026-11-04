<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  // Shared modal frame: header (title + close cross) above a padded body; width as a Tailwind class
  let { title, class: width = "w-90", onclose, children }: { title: string; class?: string; onclose: () => void; children: Snippet } = $props();

  let dialog: HTMLDialogElement;
  onMount(() => dialog.showModal());
</script>

<dialog bind:this={dialog} class={width} oncancel={(e) => (e.preventDefault(), onclose())}>
  <header>
    <h2 class="label">{title}</h2>
    <button type="button" class="btn bare icon" title="Close" aria-label="Close" onclick={onclose}><Icon name="close" /></button>
  </header>
  <div class="body">{@render children()}</div>
</dialog>

<style>
  dialog {
    max-width: 90vw;
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
  }
  .body {
    padding: calc(var(--spacing) * 4.5);
  }
</style>
