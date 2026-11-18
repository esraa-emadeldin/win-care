<script setup lang="ts">
import { computed, ref } from 'vue'
import { formatBytes } from '../format'
import { baseName, type ReviewItem } from '../stores/scan'
import FluentIcon from './FluentIcon.vue'

const props = defineProps<{ title: string; description: string; items: ReviewItem[] }>()

// Collapsed like the mockup, except when something inside needs the user's
// attention: a warning or note is never hidden behind a chevron.
const needsAttention = props.items.some(
  (i) => !i.recyclable || i.blocked !== null || i.finding.note !== null
)
const open = ref(needsAttention)

// Only items the user may pick count for the group checkbox.
const pickable = computed(() =>
  props.items.filter((i) => i.finding.selectable && i.recyclable && i.blocked === null)
)
const allSelected = computed(
  () => pickable.value.length > 0 && pickable.value.every((i) => i.selected)
)
const someSelected = computed(() => pickable.value.some((i) => i.selected) && !allSelected.value)
const bytes = computed(() => props.items.reduce((s, i) => s + i.finding.bytes, 0))

function toggleAll(e: Event) {
  const on = (e.target as HTMLInputElement).checked
  for (const i of pickable.value) i.selected = on
}
</script>

<template>
  <section class="card group">
    <header>
      <input
        type="checkbox"
        :aria-label="title"
        :checked="allSelected"
        :indeterminate="someSelected"
        :disabled="pickable.length === 0"
        @change="toggleAll"
      />
      <button class="title" type="button" :aria-expanded="open" @click="open = !open">
        <strong>{{ title }} ({{ formatBytes(bytes) }})</strong>
        <span class="desc">{{ description }}</span>
      </button>
      <span class="size">{{ formatBytes(bytes) }}</span>
      <button
        class="chevron"
        type="button"
        :aria-label="open ? `Hide ${title}` : `Show ${title}`"
        :aria-expanded="open"
        @click="open = !open"
      >
        <FluentIcon name="chevronDown" :size="20" :class="{ flip: open }" />
      </button>
    </header>
    <ul v-if="open">
      <li v-for="item in items" :key="item.finding.id">
        <input
          v-model="item.selected"
          type="checkbox"
          :aria-label="baseName(item.finding.path)"
          :disabled="!item.finding.selectable || !item.recyclable || item.blocked !== null"
        />
        <div class="name">
          <span>{{ baseName(item.finding.path) }}</span>
          <span class="path">{{ item.finding.path }}</span>
          <span v-if="item.blocked" class="chip chip-warn">{{ item.blocked }}</span>
          <span v-if="!item.recyclable" class="chip chip-warn">too large for the Recycle Bin</span>
          <span v-if="item.finding.note" class="chip chip-unknown">{{ item.finding.note }}</span>
        </div>
        <span class="size small">{{ formatBytes(item.finding.bytes) }}</span>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.group {
  padding: 4px 18px;
  border-radius: var(--radius);
}
header {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 9px 0;
}
.title {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  text-align: left;
  border: none;
  background: none;
  color: var(--text);
  padding: 0;
}
.title strong {
  font-size: 16px;
  font-weight: 600;
}
.desc {
  color: var(--text-2);
  font-size: 14px;
}
.size {
  font-size: 16px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  min-width: 80px;
  text-align: right;
}
.chevron {
  border: none;
  background: none;
  color: var(--text-2);
  padding: 4px;
  border-radius: 8px;
  display: inline-flex;
}
.chevron:hover {
  background: var(--glass-strong);
}
.flip {
  transform: rotate(180deg);
}
ul {
  list-style: none;
  margin: 0;
  padding: 4px 0 8px 36px;
  border-top: 1px solid var(--divider);
}
li {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 8px 0;
}
.name {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}
.path {
  font-size: 12px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chip {
  align-self: flex-start;
}
.size.small {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-2);
  margin-right: 34px;
}
</style>
