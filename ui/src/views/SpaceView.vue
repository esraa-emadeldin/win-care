<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { api, onSpacePartial } from '../api'
import { formatBytes } from '../format'
import { startReview } from '../stores/scan'
import type { Drive, SpaceNode } from '../types'
import SpaceTreemap from '../components/SpaceTreemap.vue'

const router = useRouter()
const drives = ref<Drive[]>([])
const drive = ref('C')
const root = ref<SpaceNode | null>(null)
const trail = ref<SpaceNode[]>([])
const selected = ref<SpaceNode | null>(null)
const scanning = ref(false)
const partial = ref<SpaceNode[]>([])
let unlisten: UnlistenFn | undefined

onMounted(async () => {
  drives.value = await api.drives()
  drive.value = drives.value[0]?.letter ?? 'C'
  unlisten = await onSpacePartial((n) => partial.value.push(n))
})
onUnmounted(() => unlisten?.())

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

async function scan() {
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
    <header>
      <div>
        <h1>Space Map</h1>
        <p class="muted">See what's using your disk space. Double-click a block to look inside.</p>
      </div>
      <div class="tools">
        <select v-model="drive" aria-label="Drive" :disabled="scanning">
          <option v-for="d in drives" :key="d.letter" :value="d.letter">
            Local Disk ({{ d.letter }}:) — {{ formatBytes(d.total - d.free) }} used of
            {{ formatBytes(d.total) }}
          </option>
        </select>
        <button v-if="!scanning" class="btn" type="button" @click="scan">Scan drive</button>
        <button v-else class="btn" type="button" @click="api.cancelSpaceScan()">Cancel</button>
      </div>
    </header>

    <nav v-if="root" class="crumbs" aria-label="Folder path">
      <button type="button" class="link" @click="up(0)">{{ root.name }}</button>
      <template v-for="(n, i) in trail" :key="n.path">
        › <button type="button" class="link" @click="up(i + 1)">{{ n.name }}</button>
      </template>
    </nav>

    <div class="card map-card">
      <SpaceTreemap
        v-if="current"
        :node="current"
        :selected="selected?.path ?? ''"
        @select="selected = $event"
        @open="open"
      />
      <p v-else class="muted empty">Pick a drive and press “Scan drive”.</p>
    </div>

    <footer v-if="selected">
      <span>
        <strong>{{ selected.name }}</strong>
        <span class="muted"> — {{ formatBytes(selected.bytes) }}</span>
      </span>
      <button class="btn" type="button" @click="api.revealPath(selected.path)">
        Open in Explorer
      </button>
      <button class="btn" type="button" @click="review">Review for deletion</button>
    </footer>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space) * 2);
  height: 100%;
}
header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}
h1 {
  font-size: 28px;
}
.tools {
  display: flex;
  gap: var(--space);
}
select {
  font: inherit;
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text);
}
.crumbs .link {
  border: none;
  background: none;
  color: var(--primary);
  padding: 0 2px;
}
.map-card {
  flex: 1;
  padding: var(--space);
  min-height: 340px;
}
.empty {
  padding: calc(var(--space) * 4);
  text-align: center;
}
footer {
  display: flex;
  align-items: center;
  gap: var(--space);
}
footer > span {
  flex: 1;
}
</style>
