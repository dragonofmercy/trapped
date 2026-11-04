<script lang="ts">
  import type { Snippet } from "svelte";

  // Button row at the bottom of a modal's content (kept inside a <form> when there is one, so Enter submits)
  let { children }: { children: Snippet } = $props();
</script>

<div>{@render children()}</div>

<style>
  div {
    display: flex;
    justify-content: flex-end;
    gap: calc(var(--spacing) * 2);
  }
</style>
