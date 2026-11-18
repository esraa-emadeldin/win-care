<script setup lang="ts">
import { computed, ref } from 'vue'
import { formatBytes } from '../format'
import { baseName, type ReviewItem } from '../stores/scan'

const props = defineProps<{ title: string; items: ReviewItem[] }>()
const open = ref(true)

// Only items the user may pick count for the group checkbox.
const pickable = computed(() =>
  props.items.filter((i) => i.finding.selectable && i.recyclable && i.blocked === null)
)
const allSelected = computed(
  () => pickable.value.length > 0 && pickable.value.every((i) => i.selected)
)
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
        :disabled="pickable.length === 0"
        @change="toggleAll"
      />
      <button class="title" type="button" :aria-expanded="open" @click="open = !open">
        {{ title }} <span class="muted">({{ items.length }})</span>
      </button>
      <span class="size">{{ formatBytes(bytes) }}</span>
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
          <span class="muted path">{{ item.finding.path }}</span>
          <span v-if="!item.recyclable" class="chip chip-warn">too large for the Recycle Bin</span>
          <span v-if="item.blocked" class="chip chip-warn">{{ item.blocked }}</span>
          <span v-if="item.finding.note" class="chip chip-unknown">{{ item.finding.note }}</span>
        </div>
        <span class="size">{{ formatBytes(item.finding.bytes) }}</span>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.group {
  padding: var(--space) calc(var(--space) * 2);
}
header,
li {
  display: flex;
  align-items: center;
  gap: calc(var(--space) * 1.5);
  padding: var(--space) 0;
}
.title {
  flex: 1;
  text-align: left;
  border: none;
  background: none;
  color: var(--text);
  font-weight: 600;
  padding: 0;
}
ul {
  list-style: none;
  margin: 0;
  padding: 0 0 0 calc(var(--space) * 3);
  border-top: 1px solid var(--border);
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
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chip {
  align-self: flex-start;
}
.size {
  font-variant-numeric: tabular-nums;
  color: var(--text-2);
  min-width: 80px;
  text-align: right;
}
input[type='checkbox'] {
  width: 18px;
  height: 18px;
  accent-color: var(--primary);
}
</style>
