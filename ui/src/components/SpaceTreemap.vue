<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { formatBytes } from '../format'
import { layout } from '../treemap'
import type { SpaceNode } from '../types'

const props = defineProps<{ node: SpaceNode; selected: string }>()
const emit = defineEmits<{ open: [SpaceNode]; select: [SpaceNode] }>()
const box = ref<HTMLDivElement>()
const size = ref({ w: 0, h: 0 })
const COLORS = ['#2563EB', '#16A34A', '#7C3AED', '#F59E0B', '#06B6D4', '#EF4444', '#64748B']

let observer: ResizeObserver | undefined
onMounted(() => {
  observer = new ResizeObserver(([e]) => {
    if (e) size.value = { w: e.contentRect.width, h: e.contentRect.height }
  })
  if (box.value) observer.observe(box.value)
})
onUnmounted(() => observer?.disconnect())

const rects = computed(() => layout(props.node, size.value.w, size.value.h))
const child = (path: string, name: string) =>
  props.node.children.find((c) => c.path === path && c.name === name)
</script>

<template>
  <div ref="box" class="map">
    <button
      v-for="(r, i) in rects"
      :key="r.path || r.name"
      type="button"
      class="tile"
      :class="{ selected: r.path !== '' && r.path === selected, other: r.path === '' }"
      :style="{
        left: `${r.x}px`,
        top: `${r.y}px`,
        width: `${r.w}px`,
        height: `${r.h}px`,
        background: COLORS[i % COLORS.length]
      }"
      :title="`${r.name}: ${formatBytes(r.bytes)}`"
      :disabled="r.path === ''"
      @click="emit('select', child(r.path, r.name)!)"
      @dblclick="emit('open', child(r.path, r.name)!)"
    >
      <span v-if="r.w > 70 && r.h > 36">{{ r.name }}<br />{{ formatBytes(r.bytes) }}</span>
    </button>
  </div>
</template>

<style scoped>
.map {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 320px;
}
.tile {
  position: absolute;
  border: 2px solid var(--surface-alt);
  border-radius: 8px;
  color: #fff;
  text-align: left;
  padding: 8px;
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  line-height: 1.3;
}
.tile.other {
  opacity: 0.55;
  cursor: default;
}
.tile.selected {
  outline: 3px solid var(--text);
  outline-offset: -3px;
}
</style>
