<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { formatBytes } from '../format'
import { spaceColor } from '../spaceColors'
import { layout } from '../treemap'
import type { SpaceNode } from '../types'

const props = defineProps<{ node: SpaceNode; selected: string }>()
const emit = defineEmits<{ open: [SpaceNode]; select: [SpaceNode] }>()
const box = ref<HTMLDivElement>()
const size = ref({ w: 0, h: 0 })
/** Space between tiles, as in the mockup. */
const GAP = 6

let observer: ResizeObserver | undefined
onMounted(() => {
  observer = new ResizeObserver(([e]) => {
    if (e) size.value = { w: e.contentRect.width, h: e.contentRect.height }
  })
  if (box.value) observer.observe(box.value)
})
onUnmounted(() => observer?.disconnect())

const rects = computed(() => layout(props.node, size.value.w + GAP, size.value.h + GAP))
const child = (path: string, name: string) =>
  props.node.children.find((c) => c.path === path && c.name === name)
const pct = (bytes: number) => (props.node.bytes ? Math.round((bytes / props.node.bytes) * 100) : 0)
</script>

<template>
  <div ref="box" class="map">
    <button
      v-for="(r, i) in rects"
      :key="r.path || r.name"
      type="button"
      class="tile"
      :class="{
        selected: r.path !== '' && r.path === selected,
        other: r.path === '',
        quiet: i >= 4
      }"
      :style="{
        left: `${r.x}px`,
        top: `${r.y}px`,
        width: `${Math.max(r.w - GAP, 0)}px`,
        height: `${Math.max(r.h - GAP, 0)}px`,
        '--tile': spaceColor(i, r.path === '')
      }"
      :title="`${r.name}: ${formatBytes(r.bytes)}`"
      :disabled="r.path === ''"
      @click="emit('select', child(r.path, r.name)!)"
      @dblclick="emit('open', child(r.path, r.name)!)"
    >
      <span v-if="r.w > 90 && r.h > 70" class="label">
        <span class="name">{{ r.name }}</span>
        <strong>{{ formatBytes(r.bytes) }}</strong>
        <span>({{ pct(r.bytes) }}%)</span>
      </span>
    </button>
  </div>
</template>

<style scoped>
.map {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 300px;
}
.tile {
  position: absolute;
  /* Label at the top left, like the mockup (buttons centre by default). */
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  border: none;
  border-radius: 10px;
  color: #fff;
  text-align: left;
  padding: 16px;
  overflow: hidden;
  background: linear-gradient(
    160deg,
    color-mix(in srgb, var(--tile) 82%, #fff) 0%,
    var(--tile) 100%
  );
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.25);
}
.tile.quiet,
.tile.other {
  opacity: 0.7;
}
.tile.other {
  cursor: default;
}
.tile:hover:not(:disabled) {
  filter: brightness(1.06);
}
.tile.selected {
  outline: 3px solid var(--text);
  outline-offset: -3px;
}
.label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 15px;
  line-height: 1.2;
}
.label strong {
  font-family: var(--font-display);
  font-size: 22px;
}
.name {
  font-weight: 500;
}
</style>
