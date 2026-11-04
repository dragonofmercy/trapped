<script lang="ts" generics="T extends string | number">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let { value = $bindable(), options, icon, label: name }: { value: T; options: { value: T; label: string; badge?: string | number }[]; icon?: Snippet<[T]>; label?: string } = $props();
  let open = $state(false);
  let root: HTMLDivElement;

  const current = $derived(options.find((o) => o.value === value));

  // The window clips anything outside it: open upward when the list does not fit below.
  let up = $state(false);
  function toggle() {
    if (!open) {
      const r = root.getBoundingClientRect();
      const need = options.length * 32 + 16;
      up = window.innerHeight - r.bottom < need && r.top > window.innerHeight - r.bottom;
    }
    open = !open;
  }

  function pick(v: T) {
    value = v;
    open = false;
  }
</script>

<svelte:window onclick={(e) => open && !root.contains(e.target as Node) && (open = false)} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<div class="select" bind:this={root}>
  <button type="button" class="field current" onclick={toggle} aria-label={name} aria-haspopup="listbox" aria-expanded={open}>
    {@render icon?.(value)}
    <span class="text">{current?.label ?? ""}</span>
    {#if current?.badge !== undefined}<span class="badge">{current.badge}</span>{/if}
    <span class="caret" class:open><Icon name="chevron" /></span>
  </button>
  {#if open}
    <ul role="listbox" class:up>
      {#each options as o (o.value)}
        <li role="option" aria-selected={o.value === value}>
          <button type="button" class:active={o.value === value} onclick={() => pick(o.value)}>
            {@render icon?.(o.value)}<span class="text">{o.label}</span>
            {#if o.badge !== undefined}<span class="badge">{o.badge}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select {
    position: relative;
    min-width: 0;
  }
  .current {
    width: 100%;
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 2.5);
    padding: 0 calc(var(--spacing) * 2.5) 0 calc(var(--spacing) * 3);
    cursor: pointer;
    text-align: left;
  }
  .text {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .badge {
    font:
      600 var(--text-xs) var(--font-mono);
    color: var(--muted);
    background: rgba(127, 127, 127, 0.14);
    border-radius: var(--radius-xl);
    padding: calc(var(--spacing) * 0.25) calc(var(--spacing) * 2);
  }
  .caret {
    display: flex;
    color: var(--muted);
    transition: transform 0.15s;
  }
  .caret.open {
    transform: rotate(180deg);
  }
  ul {
    position: absolute;
    z-index: 20;
    left: 0;
    top: calc(var(--spacing) * 9.5);
    max-width: calc(100vw - calc(var(--spacing) * 8));
    max-height: 60vh;
    overflow-y: auto;
    min-width: 100%;
    width: max-content;
    margin: 0;
    padding: calc(var(--spacing) * 1.25);
    list-style: none;
    background: var(--dialog-bg);
    border: 1px solid var(--line2);
    border-radius: var(--radius-md);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.55);
  }
  ul.up {
    top: auto;
    bottom: calc(var(--spacing) * 9.5);
  }
  ul button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: calc(var(--spacing) * 2.5);
    height: calc(var(--spacing) * 8);
    padding: 0 calc(var(--spacing) * 2.5);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    font:
      500 var(--text-sm) var(--font-sans);
    cursor: pointer;
    white-space: nowrap;
    text-align: left;
  }
  ul button:hover {
    background: rgba(var(--accent-rgb), 0.16);
  }
  ul button.active {
    background: rgba(var(--accent-rgb), 0.24);
  }
</style>
