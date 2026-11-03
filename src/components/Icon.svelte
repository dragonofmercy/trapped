<script lang="ts" module>
  // Same icon set and stroke style as Wallo
  const PATHS = {
    plus: "M8 3v10M3 8h10",
    pencil: "M10.5 2.5l3 3-8 8H2.5v-3z",
    trash: "M2.5 4.5h11M6.5 4.5V3h3v1.5M4 4.5l.7 9h6.6l.7-9",
    reset: "M3 8a5 5 0 1 0 1.5-3.5M3 2.5v3h3",
    check: "M3 8.5l3.2 3L13 4.5",
    close: "M4 4l8 8M12 4l-8 8",
    download: "M8 2.5v8M4.5 7.5L8 11l3.5-3.5M3 13.5h10",
    chevron: "M4 6l4 4 4-4",
    external: "M9 2.5h4.5V7M13.5 2.5L7 9M11.5 9.5v3a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1h3",
    sliders: "M2 4.5h12M2 11.5h12M5.5 3v3M10.5 10v3",
    inbox: "M1.5 9l2-5.5h9l2 5.5v3.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1zM1.5 9h4l1 1.5h3l1-1.5h4",
    clip: "M13.5 7.5l-5.6 5.6a3.2 3.2 0 0 1-4.5-4.5l5.8-5.8a2.1 2.1 0 0 1 3 3l-5.8 5.8a1 1 0 0 1-1.5-1.5L10.3 4.7",
    monitor: "M2.5 3h11a1 1 0 0 1 1 1v6.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM6 14h4M8 11.5V14",
    phone: "M5 1.5h6a1 1 0 0 1 1 1v11a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1zM7.5 12.5h1",
  };

  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  let { name, size = 15 }: { name: IconName; size?: number } = $props();
</script>

<svg viewBox="0 0 16 16" width={size} height={size} aria-hidden="true"><path d={PATHS[name]} /></svg>

<style>
  svg {
    flex-shrink: 0;
    stroke: currentColor;
    fill: none;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
