<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from '../api'
import { formatBytes } from '../format'
import type { Vitals } from '../types'
import SparkLine from '../components/SparkLine.vue'
import FluentIcon from '../components/FluentIcon.vue'

/** Samples while the popup is visible; 30 samples at 2 s = the last minute. */
const INTERVAL_MS = 2000
const KEEP = 30
/** Above this share of the system disk in use, the tip says so. */
const DISK_FULL_PCT = 80

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
const diskPct = computed(() => (now.value ? pct(now.value.diskUsed, now.value.diskTotal) : 0))

// The tray is its own window and does not see the main window's scan, so the
// tip is built from what it measures itself.
const tip = computed(() => {
  if (!now.value) return null
  const free = formatBytes(now.value.diskTotal - now.value.diskUsed)
  return diskPct.value >= DISK_FULL_PCT
    ? { title: 'Your disk is getting full', text: `Only ${free} left. Run a Smart Scan.` }
    : { title: 'Free up more space', text: 'Run a Smart Scan to find junk files.' }
})

/** Shows the main window, optionally on a given page, and closes the popup. */
async function openApp(path?: string) {
  try {
    const { Window } = await import('@tauri-apps/api/window')
    const { emitTo } = await import('@tauri-apps/api/event')
    if (path) await emitTo('main', 'navigate', path)
    const main = await Window.getByLabel('main')
    await main?.show()
    await main?.setFocus()
    await (await Window.getByLabel('tray'))?.hide()
  } catch {
    // Browser preview: there is no main window.
  }
}
</script>

<template>
  <div class="tray">
    <header>
      <span class="mark"><FluentIcon name="shield" :size="32" /></span>
      <strong>win-care</strong>
      <button class="gear" type="button" aria-label="Settings" @click="openApp('/settings')">
        <FluentIcon name="settings" :size="22" />
      </button>
    </header>

    <div v-if="now" class="grid">
      <div class="metric">
        <span class="label">CPU</span>
        <div class="row">
          <strong>{{ Math.round(now.cpuPct) }}%</strong>
          <SparkLine :values="history.map((h) => h.cpuPct)" :max="100" color="#60a5fa" />
        </div>
      </div>
      <div class="metric">
        <span class="label">RAM</span>
        <div class="row">
          <strong>{{ pct(now.ramUsed, now.ramTotal) }}%</strong>
          <SparkLine
            :values="history.map((h) => pct(h.ramUsed, h.ramTotal))"
            :max="100"
            color="#4ade80"
          />
        </div>
      </div>
      <div class="metric">
        <span class="label">Disk</span>
        <div class="row">
          <strong>{{ diskPct }}%</strong>
          <SparkLine
            :values="history.map((h) => pct(h.diskUsed, h.diskTotal))"
            :max="100"
            color="#c084fc"
          />
        </div>
      </div>
      <div v-if="now.batteryPct !== null" class="metric">
        <span class="label">Battery</span>
        <div class="row">
          <strong>{{ now.batteryPct }}%</strong>
          <span
            class="battery"
            :class="{ low: now.batteryPct < 20 }"
            :title="now.charging ? 'Charging' : 'On battery'"
          >
            <span class="fill" :style="{ width: `${now.batteryPct}%` }" />
          </span>
        </div>
      </div>
      <div class="metric wide">
        <span class="label">Network</span>
        <div class="row net">
          <FluentIcon name="wifi" :size="22" />
          <strong>{{ formatBytes(now.netRxBps) }}/s</strong>
        </div>
      </div>
    </div>
    <p v-else class="muted">Reading…</p>

    <button v-if="tip" class="card tip" type="button" @click="openApp('/')">
      <span class="bulb"><FluentIcon name="bulb" :size="30" /></span>
      <span class="tip-text">
        <strong>{{ tip.title }}</strong>
        <span>{{ tip.text }}</span>
      </span>
      <FluentIcon name="chevronRight" :size="20" class="chev" />
    </button>

    <button class="open" type="button" @click="openApp()">Open app</button>
  </div>
</template>

<style scoped>
.tray {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 18px 18px 16px;
}
header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 21px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--divider);
}
.mark {
  color: var(--primary);
  display: inline-flex;
}
.gear {
  margin-left: auto;
  border: none;
  background: none;
  color: var(--text);
  padding: 4px;
  border-radius: 8px;
  display: inline-flex;
}
.gear:hover {
  background: var(--glass-strong);
}
.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px 18px;
}
.metric {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--divider);
}
.metric.wide {
  grid-column: span 2;
  border-bottom: none;
  padding-bottom: 0;
}
.label {
  font-size: 17px;
  color: var(--text);
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.row strong {
  font-family: var(--font-display);
  font-size: 27px;
  font-weight: 700;
}
.row.net {
  justify-content: flex-start;
}
.row.net strong {
  font-size: 23px;
}
.battery {
  position: relative;
  width: 40px;
  height: 18px;
  border: 2px solid #22c55e;
  border-radius: 4px;
  padding: 2px;
}
.battery::after {
  content: '';
  position: absolute;
  right: -5px;
  top: 4px;
  width: 3px;
  height: 6px;
  border-radius: 0 2px 2px 0;
  background: #22c55e;
}
.battery .fill {
  display: block;
  height: 100%;
  border-radius: 2px;
  background: #22c55e;
}
.battery.low {
  border-color: var(--error);
}
.battery.low .fill,
.battery.low::after {
  background: var(--error);
}
.tip {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  border-radius: var(--radius);
  text-align: left;
  color: var(--text);
}
.bulb {
  color: #facc15;
  display: inline-flex;
}
.tip-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 14.5px;
}
.tip-text strong {
  font-size: 16.5px;
}
.chev {
  color: var(--primary);
}
.open {
  margin-top: auto;
  background: var(--primary-gradient);
  color: #fff;
  border: none;
  border-radius: 10px;
  padding: 13px;
  font-size: 17px;
  font-weight: 600;
  box-shadow: 0 4px 14px rgb(37 99 235 / 0.3);
}
</style>
