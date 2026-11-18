<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ values: number[]; max?: number; color?: string }>()
const W = 80
const H = 24

const points = computed(() => {
  const v = props.values
  if (v.length < 2) return ''
  const top = props.max ?? Math.max(...v, 1)
  return v
    .map((n, i) => `${(i / (v.length - 1)) * W},${H - (Math.min(n, top) / top) * (H - 2) - 1}`)
    .join(' ')
})
</script>

<template>
  <svg :viewBox="`0 0 ${W} ${H}`" :width="W" :height="H" aria-hidden="true">
    <polyline
      :points="points"
      fill="none"
      :stroke="color ?? 'var(--accent)'"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>
