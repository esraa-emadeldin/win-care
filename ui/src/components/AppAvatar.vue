<script setup lang="ts">
import { computed } from 'vue'

// A round coloured initial stands in for the app's own icon (reading real
// icons from Windows is planned for later).
const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 36 })
const COLORS = ['#2563eb', '#7c3aed', '#16a34a', '#0891b2', '#db2777', '#ea580c', '#4f46e5']

const color = computed(() => {
  let h = 0
  for (const c of props.name) h = (h * 31 + c.charCodeAt(0)) >>> 0
  return COLORS[h % COLORS.length]
})
const initial = computed(() => props.name.trim().charAt(0).toUpperCase() || '?')
</script>

<template>
  <span
    class="avatar"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      background: color,
      fontSize: `${size * 0.45}px`
    }"
    aria-hidden="true"
    >{{ initial }}</span
  >
</template>

<style scoped>
.avatar {
  display: inline-grid;
  place-items: center;
  flex: none;
  border-radius: 50%;
  color: #fff;
  font-weight: 700;
  font-family: var(--font-display);
  box-shadow: inset 0 -2px 4px rgb(0 0 0 / 0.15);
}
</style>
