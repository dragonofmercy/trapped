<script lang="ts">
  // Mail view placeholder when no mail is selected: open envelope illustration + hint
</script>

<div class="m-auto flex flex-col items-center gap-1 text-center select-none">
  <svg viewBox="0 0 120 104" class="mb-5 w-36" aria-hidden="true">
    <defs>
      <linearGradient id="ns-flap" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#a78bfa" />
        <stop offset="1" stop-color="var(--accent)" />
      </linearGradient>
      <linearGradient id="ns-letter" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#f1f5f9" />
        <stop offset="1" stop-color="#bfdcf7" />
      </linearGradient>
      <linearGradient id="ns-front" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#1f4f82" />
        <stop offset="1" stop-color="var(--bg2)" />
      </linearGradient>
      <radialGradient id="ns-glow">
        <stop offset="0" stop-color="var(--cyan)" stop-opacity="0.35" />
        <stop offset="1" stop-color="var(--cyan)" stop-opacity="0" />
      </radialGradient>
    </defs>
    <ellipse cx="60" cy="96" rx="52" ry="8" fill="url(#ns-glow)" />
    <path d="M10 46 L60 8 L110 46 Z" fill="url(#ns-flap)" />
    <rect x="10" y="44" width="100" height="48" rx="6" fill="var(--bg1)" />
    <rect x="26" y="18" width="68" height="56" rx="4" fill="url(#ns-letter)" />
    <g stroke="#94b4d8" stroke-width="2.5" stroke-linecap="round">
      <line x1="36" y1="32" x2="70" y2="32" />
      <line x1="36" y1="41" x2="84" y2="41" />
      <line x1="36" y1="50" x2="78" y2="50" />
    </g>
    <path d="M10 46 L60 76 L110 46 V86 A6 6 0 0 1 104 92 H16 A6 6 0 0 1 10 86 Z" fill="url(#ns-front)" stroke="var(--line2)" stroke-linejoin="round" />
  </svg>
  <p class="font-semibold text-text">Select a mail to read it</p>
  <p class="text-muted">No mail selected</p>
</div>
