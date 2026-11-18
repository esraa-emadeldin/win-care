<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { api, onSpacePartial } from '../api'
import { formatBytes } from '../format'
import { spaceColor } from '../spaceColors'
import { startReview } from '../stores/scan'
import type { Drive, SpaceNode } from '../types'
import SpaceTreemap from '../components/SpaceTreemap.vue'
import FluentIcon from '../components/FluentIcon.vue'

/** Donut geometry (SVG units). */
const R = 42
const CIRC = 2 * Math.PI * R
/** Named slices in the donut legend; the rest is "Others". */
const LEGEND_SLICES = 3

const router = useRouter()
const drives = ref<Drive[]>([])
const drive = ref('')
const root = ref<SpaceNode | null>(null)
const trail = ref<SpaceNode[]>([])
const selected = ref<SpaceNode | null>(null)
const scanning = ref(false)
const partial = ref<SpaceNode[]>([])
let unlisten: UnlistenFn | undefined

// Like the mockup: the map fills in as soon as the page opens.
onMounted(async () => {
  unlisten = await onSpacePartial((n) => partial.value.push(n))
  drives.value = await api.drives()
  drive.value = drives.value[0]?.letter ?? 'C'
})
onUnmounted(() => {
  unlisten?.()
  if (scanning.value) api.cancelSpaceScan()
})
watch(drive, (letter) => letter && scan())

const current = computed<SpaceNode | null>(() => {
  if (trail.value.length) return trail.value[trail.value.length - 1]!
  if (root.value) return root.value
  if (!scanning.value) return null
  // While scanning, draw what has been measured so far.
  const children = [...partial.value].sort((a, b) => b.bytes - a.bytes)
  return {
    name: `${drive.value}:\\`,
    path: `${drive.value}:\\`,
    bytes: children.reduce((s, c) => s + c.bytes, 0),
    children
  }
})

const info = computed(() => drives.value.find((d) => d.letter === drive.value))

/** The donut: the biggest folders of the drive, then everything else. */
const slices = computed(() => {
  const node = root.value ?? current.value
  if (!node || node.bytes === 0) return []
  const top = node.children.filter((c) => c.path !== '').slice(0, LEGEND_SLICES)
  const rest = node.bytes - top.reduce((s, c) => s + c.bytes, 0)
  const parts = top.map((c, i) => ({ name: c.name, bytes: c.bytes, color: spaceColor(i) }))
  if (rest > 0) parts.push({ name: 'Others', bytes: rest, color: spaceColor(LEGEND_SLICES) })
  let offset = 0
  return parts.map((p) => {
    const share = p.bytes / node.bytes
    const arc = { ...p, pct: Math.round(share * 100), dash: share * CIRC, offset }
    offset += share * CIRC
    return arc
  })
})

async function scan() {
  if (scanning.value) await api.cancelSpaceScan()
  scanning.value = true
  root.value = null
  trail.value = []
  selected.value = null
  partial.value = []
  try {
    root.value = await api.spaceScan(`${drive.value}:\\`)
  } finally {
    scanning.value = false
  }
}

function open(node: SpaceNode) {
  if (node.children.length) {
    trail.value.push(node)
    selected.value = null
  }
}

function up(index: number) {
  trail.value = trail.value.slice(0, index)
  selected.value = null
}

function review() {
  if (!selected.value) return
  startReview('space', [
    {
      id: selected.value.path,
      category: 'spaceMap',
      path: selected.value.path,
      bytes: selected.value.bytes,
      isDir: selected.value.children.length > 0,
      modifiedMs: 0,
      preselected: true,
      selectable: true,
      note: null
    }
  ])
  router.push('/review')
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h1>Space Map</h1>
        <p class="subtitle">See what's using your disk space.</p>
      </div>
      <div class="tools">
        <label class="field drive">
          <select v-model="drive" aria-label="Drive">
            <option v-for="d in drives" :key="d.letter" :value="d.letter">
              Local Disk ({{ d.letter }}:)
            </option>
          </select>
          <FluentIcon name="chevronDown" :size="16" />
        </label>
        <button v-if="scanning" class="btn" type="button" @click="api.cancelSpaceScan()">
          Cancel
        </button>
      </div>
    </div>

    <nav v-if="trail.length" class="crumbs" aria-label="Folder path">
      <button type="button" class="link" @click="up(0)">{{ root?.name }}</button>
      <template v-for="(n, i) in trail" :key="n.path">
        › <button type="button" class="link" @click="up(i + 1)">{{ n.name }}</button>
      </template>
    </nav>

    <div class="body">
      <div class="map-wrap">
        <SpaceTreemap
          v-if="current"
          :node="current"
          :selected="selected?.path ?? ''"
          @select="selected = $event"
          @open="open"
        />
        <p v-else class="muted empty">Reading the drive…</p>
      </div>

      <aside class="card summary">
        <div class="donut-row">
          <svg viewBox="0 0 100 100" width="104" height="104" aria-hidden="true">
            <circle cx="50" cy="50" :r="R" class="track" />
            <circle
              v-for="s in slices"
              :key="s.name"
              cx="50"
              cy="50"
              :r="R"
              class="slice"
              :stroke="s.color"
              :stroke-dasharray="`${s.dash} ${CIRC}`"
              :stroke-dashoffset="-s.offset"
            />
          </svg>
          <div v-if="info">
            <p class="used">{{ formatBytes(info.total - info.free) }}</p>
            <p class="of">Used of {{ formatBytes(info.total) }}</p>
          </div>
        </div>
        <ul class="legend">
          <li v-for="s in slices" :key="s.name">
            <span class="dot" :style="{ background: s.color }" />
            <span class="lname">{{ s.name }}</span>
            <span>{{ s.pct }}%</span>
          </li>
        </ul>
      </aside>
    </div>

    <footer v-if="selected" class="card selection">
      <span>
        <strong>{{ selected.name }}</strong>
        <span class="muted"> — {{ formatBytes(selected.bytes) }}</span>
      </span>
      <button class="btn" type="button" @click="api.revealPath(selected.path)">
        Open in Explorer
      </button>
      <button class="btn btn-primary" type="button" @click="review">Review for deletion</button>
    </footer>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
  max-width: 1080px;
}
.tools {
  display: flex;
  gap: 10px;
  align-items: center;
}
.drive {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  padding: 9px 14px;
}
.drive select {
  appearance: none;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--text);
  cursor: pointer;
}
.crumbs .link {
  border: none;
  background: none;
  color: var(--primary);
  padding: 0 2px;
  font-weight: 600;
}
.body {
  flex: 1;
  min-height: 320px;
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 18px;
}
.map-wrap {
  min-height: 300px;
}
.empty {
  padding: 32px;
  text-align: center;
}
.summary {
  padding: 22px 24px;
  align-self: start;
}
.donut-row {
  display: flex;
  align-items: center;
  gap: 18px;
}
.track {
  fill: none;
  stroke: var(--divider);
  stroke-width: 12;
}
.slice {
  fill: none;
  stroke-width: 12;
  transform: rotate(-90deg);
  transform-origin: 50% 50%;
}
.used {
  margin: 0;
  font-family: var(--font-display);
  font-size: 30px;
  font-weight: 700;
}
.of {
  margin: 0;
  color: var(--text-2);
  font-size: 15px;
}
.legend {
  list-style: none;
  padding: 0;
  margin: 18px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 15.5px;
}
.legend li {
  display: flex;
  align-items: center;
  gap: 12px;
}
.dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}
.lname {
  flex: 1;
}
.selection {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
}
.selection > span {
  flex: 1;
}
</style>
