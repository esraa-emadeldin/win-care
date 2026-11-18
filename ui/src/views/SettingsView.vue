<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { api } from '../api'
import { formatBytes } from '../format'
import { applyTheme } from '../theme'
import type { ActivityEntry, Settings } from '../types'
import ToggleSwitch from '../components/ToggleSwitch.vue'

const settings = ref<Settings | null>(null)
const log = ref<ActivityEntry[]>([])
const error = ref('')

onMounted(async () => {
  ;[settings.value, log.value] = await Promise.all([api.getSettings(), api.activity(50)])
})

// Save every change straight away; there is no "Apply" button to forget.
watch(
  settings,
  async (s, old) => {
    if (!s || !old) return
    error.value = ''
    try {
      await api.setSettings(s)
      applyTheme(s.theme)
    } catch (e) {
      error.value = String(e)
    }
  },
  { deep: true }
)

const when = (ms: number) => new Date(ms).toLocaleString()
</script>

<template>
  <div v-if="settings" class="page">
    <header>
      <h1>Settings</h1>
    </header>
    <p v-if="error" role="alert" class="error">{{ error }}</p>

    <section class="card panel">
      <div class="row">
        <span>Start with Windows</span>
        <ToggleSwitch v-model="settings.startWithWindows" label="Start with Windows" />
      </div>
      <div class="row">
        <label for="theme">Theme</label>
        <select id="theme" v-model="settings.theme">
          <option value="system">Same as Windows</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </div>
      <div class="row">
        <label for="low-disk">Warn when free space is below</label>
        <select id="low-disk" v-model.number="settings.lowDiskAlertPct">
          <option v-for="p in [5, 10, 15, 20]" :key="p" :value="p">{{ p }} %</option>
        </select>
      </div>
      <div class="row">
        <span>
          Check for updates once a day
          <br /><span class="muted small">The only time win-care uses the internet.</span>
        </span>
        <ToggleSwitch v-model="settings.checkUpdates" label="Check for updates once a day" />
      </div>
    </section>

    <section class="card panel">
      <h2>Activity</h2>
      <p v-if="!log.length" class="muted">Nothing cleaned yet.</p>
      <ul>
        <li v-for="e in log" :key="e.atMs">
          <span>{{ when(e.atMs) }}</span>
          <span
            >Moved {{ e.done }} items to the Recycle Bin, freed
            {{ formatBytes(e.freedBytes) }}</span
          >
          <span v-if="e.skipped" class="muted">{{ e.skipped }} skipped</span>
        </li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space) * 2);
  max-width: 720px;
}
h1 {
  font-size: 28px;
}
h2 {
  font-size: 17px;
}
.panel {
  padding: calc(var(--space) * 2.5);
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: calc(var(--space) * 2);
  padding: 12px 0;
  border-bottom: 1px solid var(--border);
}
.row:last-child {
  border-bottom: none;
}
select {
  font: inherit;
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text);
}
.small {
  font-size: 12px;
}
ul {
  list-style: none;
  padding: 0;
}
li {
  display: flex;
  gap: calc(var(--space) * 2);
  padding: 6px 0;
}
.error {
  color: var(--error);
}
</style>
