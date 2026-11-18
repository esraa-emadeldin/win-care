<script setup lang="ts">
// The four colourful Smart Scan illustrations from the mockup, as gradient SVGs.
defineProps<{ name: 'broom' | 'apps' | 'rocket' | 'heart' }>()
</script>

<template>
  <svg viewBox="0 0 64 64" width="56" height="56" aria-hidden="true">
    <defs>
      <linearGradient id="g-blue" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#60a5fa" />
        <stop offset="1" stop-color="#1d4ed8" />
      </linearGradient>
      <linearGradient id="g-green" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#4ade80" />
        <stop offset="1" stop-color="#16a34a" />
      </linearGradient>
      <linearGradient id="g-purple" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#c084fc" />
        <stop offset="1" stop-color="#6d28d9" />
      </linearGradient>
      <linearGradient id="g-teal" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#5eead4" />
        <stop offset="1" stop-color="#0d9488" />
      </linearGradient>
    </defs>

    <g v-if="name === 'broom'">
      <rect
        x="37"
        y="4"
        width="7"
        height="28"
        rx="3.5"
        fill="url(#g-blue)"
        transform="rotate(30 40 18)"
      />
      <path d="M22 30 L38 38 L30 58 C24 58 14 54 10 50 Z" fill="url(#g-blue)" />
      <path
        d="M14 50 L22 40 M19 53 L26 42 M24 56 L30 44"
        stroke="#dbeafe"
        stroke-width="2"
        stroke-linecap="round"
      />
      <rect
        x="21"
        y="27"
        width="19"
        height="7"
        rx="3"
        fill="#1e40af"
        transform="rotate(27 30 30)"
      />
    </g>

    <g v-else-if="name === 'apps'">
      <rect x="8" y="8" width="21" height="21" rx="6" fill="url(#g-green)" />
      <rect x="35" y="8" width="21" height="21" rx="6" fill="url(#g-green)" />
      <rect x="8" y="35" width="21" height="21" rx="6" fill="url(#g-green)" />
      <rect x="35" y="35" width="21" height="21" rx="6" fill="url(#g-green)" />
    </g>

    <g v-else-if="name === 'rocket'">
      <path
        d="M44 8 C54 8 56 10 56 20 C56 30 46 40 36 46 L18 28 C24 18 34 8 44 8 Z"
        fill="url(#g-purple)"
      />
      <circle cx="42" cy="22" r="5" fill="#ede9fe" />
      <path d="M18 28 L8 30 L14 20 L24 18 Z M36 46 L34 56 L44 50 L46 40 Z" fill="#8b5cf6" />
      <path d="M20 40 L10 54 L24 44 Z" fill="#f472b6" />
    </g>

    <g v-else>
      <path
        d="M32 56 C14 44 6 34 6 23 C6 14 13 8 21 8 C26 8 30 11 32 15 C34 11 38 8 43 8 C51 8 58 14 58 23 C58 34 50 44 32 56 Z"
        fill="url(#g-teal)"
      />
      <path
        d="M12 29 H23 L27 21 L33 37 L37 27 H52"
        fill="none"
        stroke="#fff"
        stroke-width="3.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </g>
  </svg>
</template>
