<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from '../api'
import { formatBytes } from '../format'
import { store } from '../stores/scan'
import type { Vitals } from '../types'
import SparkLine from '../components/SparkLine.vue'

/** Samples while the popup is visible; 30 samples at 2 s = the last minute. */
const INTERVAL_MS = 2000
const KEEP = 30

const now = ref<Vitals | null>(null)
const history = ref<Vitals[]>([])
let timer: number | undefined

async function sample() {
  try {
    const v = await api.vitals()
    now.value = v
    history.value = [...history.value, v].slice(-KEEP)
  } catch {
    // Keep the last numbers if one sample fails.
  }
}

function start() {
  if (timer === undefined) {
    sample()
    timer = window.setInterval(sample, INTERVAL_MS)
  }
}
function stop() {
  window.clearInterval(timer)
  timer = undefined
}
// Only sample while someone can see it: no work when the popup is hidden.
function onVisibility() {
  if (document.hidden) stop()
  else start()
}

onMounted(() => {
  start()
  document.addEventListener('visibilitychange', onVisibility)
})
onUnmounted(() => {
  stop()
  document.removeEventListener('visibilitychange', onVisibility)
})

const pct = (used: number, total: number) => (total ? Math.round((used / total) * 100) : 0)
const tip = computed(() => {
  const bytes = store.scan?.cleanupBytes ?? 0
  return bytes > 0 ? `Free up ${formatBytes(bytes)}` : null
})

async function openApp() {
  const { Window } = await import('@tauri-apps/api/window')
  const main = await Window.getByLabel('main')
  await main?.show()
  await main?.setFocus()
  await (await Window.getByLabel('tray'))?.hide()
}
</script>

<template>
  <div class="tray">
    <header>
      <img src="/icon.svg" alt="" width="22" height="22" />
      <strong>win-care</strong>
    </header>

    <div v-if="now" class="grid">
      <div class="metric">
        <span class="muted">CPU</span>
        <strong>{{ Math.round(now.cpuPct) }}%</strong>
        <SparkLine :values="history.map((h) => h.cpuPct)" :max="100" />
      </div>
      <div class="metric">
        <span class="muted">RAM</span>
        <strong>{{ pct(now.ramUsed, now.ramTotal) }}%</strong>
        <SparkLine
          :values="history.map((h) => pct(h.ramUsed, h.ramTotal))"
          :max="100"
          color="var(--success)"
        />
      </div>
      <div class="metric">
        <span class="muted">Disk</span>
        <strong>{{ pct(now.diskUsed, now.diskTotal) }}%</strong>
        <span class="muted small">{{ formatBytes(now.diskTotal - now.diskUsed) }} free</span>
      </div>
      <div v-if="now.batteryPct !== null" class="metric">
        <span class="muted">Battery</span>
        <strong>{{ now.batteryPct }}%</strong>
        <span class="muted small">{{ now.charging ? 'Charging' : 'On battery' }}</span>
      </div>
      <div class="metric wide">
        <span class="muted">Network</span>
        <strong>↓ {{ formatBytes(now.netRxBps) }}/s · ↑ {{ formatBytes(now.netTxBps) }}/s</strong>
      </div>
    </div>
    <p v-else class="muted">Reading…</p>

    <div v-if="tip" class="card tip">💡 {{ tip }}</div>

    <button class="open" type="button" @click="openApp">Open app</button>
  </div>
</template>

<style scoped>
.tray {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space) * 1.5);
  padding: calc(var(--space) * 2);
  background: var(--surface);
}
header {
  display: flex;
  align-items: center;
  gap: var(--space);
  font-size: 15px;
}
.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: calc(var(--space) * 2);
}
.metric {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.metric strong {
  font-family: var(--font-display);
  font-size: 22px;
}
.metric.wide {
  grid-column: span 2;
}
.metric.wide strong {
  font-size: 15px;
}
.small {
  font-size: 12px;
}
.tip {
  padding: 12px;
}
.open {
  margin-top: auto;
  background: var(--primary);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 10px;
  font-weight: 600;
}
</style>
