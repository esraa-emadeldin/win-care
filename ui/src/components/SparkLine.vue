<script setup lang="ts">
import { computed } from 'vue'

// A smooth little trend line, like the mockup's. It scales to the values' own
// range, so a small change still shows as a gentle wave.
const props = defineProps<{ values: number[]; max?: number; color?: string }>()
const W = 84
const H = 28
const PAD = 3
/** The smallest range drawn, so a perfectly steady value stays a calm line. */
const MIN_RANGE = 4

const path = computed(() => {
  const v = props.values
  if (v.length < 2) return ''
  let lo = Math.min(...v)
  let hi = Math.max(...v)
  if (hi - lo < MIN_RANGE) {
    const mid = (hi + lo) / 2
    lo = mid - MIN_RANGE / 2
    hi = mid + MIN_RANGE / 2
  }
  const pts = v.map((n, i) => ({
    x: (i / (v.length - 1)) * W,
    y: H - PAD - ((n - lo) / (hi - lo)) * (H - 2 * PAD)
  }))
  // Curve through the midpoints between samples: smooth, no overshoot.
  let d = `M ${pts[0]!.x} ${pts[0]!.y}`
  for (let i = 1; i < pts.length; i++) {
    const p = pts[i - 1]!
    const q = pts[i]!
    d += ` Q ${p.x} ${p.y} ${(p.x + q.x) / 2} ${(p.y + q.y) / 2}`
  }
  const last = pts[pts.length - 1]!
  return `${d} T ${last.x} ${last.y}`
})
</script>

<template>
  <svg :viewBox="`0 0 ${W} ${H}`" :width="W" :height="H" aria-hidden="true">
    <path
      :d="path"
      fill="none"
      :stroke="color ?? 'var(--accent)'"
      stroke-width="2.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>
