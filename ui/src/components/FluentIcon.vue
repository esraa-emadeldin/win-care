<script setup lang="ts">
import { ICONS, type IconName } from '../icons'

withDefaults(defineProps<{ name: IconName; size?: number }>(), { size: 20 })
</script>

<template>
  <!-- eslint-disable vue/no-v-html -- trusted SVG files bundled at build time -->
  <span
    class="icon"
    :style="{ width: `${size}px`, height: `${size}px` }"
    aria-hidden="true"
    v-html="ICONS[name]"
  />
</template>

<style scoped>
.icon {
  display: inline-flex;
  flex: none;
}
.icon :deep(svg) {
  width: 100%;
  height: 100%;
  fill: currentColor;
}
</style>
