<script lang="ts" module>
  import { ChevronDown, Download, ExternalLink, Inbox, MailPen, MailPlus, Monitor, Paperclip, Pencil, Plug, RotateCcw, Search, Settings, Smartphone, Trash, X } from "@lucide/svelte";

  // Lucide icons by short name, so call sites stay one-liners with a shared size and stroke
  const ICONS = {
    mailplus: MailPlus,
    pencil: Pencil,
    plug: Plug,
    trash: Trash,
    reset: RotateCcw,
    close: X,
    download: Download,
    chevron: ChevronDown,
    external: ExternalLink,
    search: Search,
    gear: Settings,
    inbox: Inbox,
    clip: Paperclip,
    monitor: Monitor,
    phone: Smartphone,
    mailpen: MailPen,
  };

  export type IconName = keyof typeof ICONS;
</script>

<script lang="ts">
  let { name }: { name: IconName } = $props();
  const Component = $derived(ICONS[name]);
</script>

<Component strokeWidth={2} class="size-4 shrink-0" aria-hidden="true" />
