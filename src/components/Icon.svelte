<script lang="ts" module>
  import { ChevronDown, Download, ExternalLink, Inbox, Monitor, Paperclip, Pencil, Plus, RotateCcw, Search, Settings, Smartphone, Trash2, X } from "@lucide/svelte";

  // Lucide icons by short name, so call sites stay one-liners with a shared size and stroke
  const ICONS = {
    plus: Plus,
    pencil: Pencil,
    trash: Trash2,
    reset: RotateCcw,
    close: X,
    download: Download,
    chevron: ChevronDown,
    external: ExternalLink,
    gear: Settings,
    search: Search,
    inbox: Inbox,
    clip: Paperclip,
    monitor: Monitor,
    phone: Smartphone,
  };

  export type IconName = keyof typeof ICONS;
</script>

<script lang="ts">
  let { name }: { name: IconName } = $props();
  const Component = $derived(ICONS[name]);
</script>

<Component size="1.25rem" strokeWidth={2} class="shrink-0" aria-hidden="true" />
