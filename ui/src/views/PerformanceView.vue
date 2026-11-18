<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import { formatBytes } from '../format'
import type { Check, StartupItem } from '../types'
import ToggleSwitch from '../components/ToggleSwitch.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'

const startup = ref<StartupItem[]>([])
const checks = ref<Check[]>([])
const binBytes = ref(0)
const confirmEmpty = ref(false)
const message = ref('')

onMounted(async () => {
  ;[startup.value, checks.value, binBytes.value] = await Promise.all([
    api.listStartup(),
    api.health(),
    api.recycleBinSize()
  ])
})

async function setEnabled(item: StartupItem, enabled: boolean) {
  try {
    await api.setStartup(item.id, enabled)
    item.enabled = enabled
  } catch (e) {
    message.value = String(e)
  }
}

function readOnlyNote(item: StartupItem): string {
  return item.source === 'task'
    ? 'Scheduled task — open Task Scheduler'
    : 'Needs admin — change it in Task Manager'
}

async function run(task: 'restartExplorer' | 'emptyRecycleBin') {
  confirmEmpty.value = false
  try {
    message.value = await api.runMaintenance(task)
    binBytes.value = await api.recycleBinSize()
  } catch (e) {
    message.value = String(e)
  }
}
</script>

<template>
  <div class="page">
    <header>
      <h1>Performance</h1>
      <p class="muted">Keep your PC running smoothly.</p>
    </header>
    <p v-if="message" role="status" class="status">{{ message }}</p>

    <div class="columns">
      <section class="card panel">
        <h2>Startup apps</h2>
        <p class="muted">Control what runs when you sign in.</p>
        <ul>
          <li v-for="item in startup" :key="item.id">
            <div class="name">
              <strong>{{ item.name }}</strong>
              <span v-if="!item.canToggle" class="muted small">{{ readOnlyNote(item) }}</span>
            </div>
            <ToggleSwitch
              v-if="item.canToggle"
              :model-value="item.enabled"
              :label="item.name"
              @update:model-value="setEnabled(item, $event)"
            />
            <button
              v-else-if="item.source === 'task'"
              class="btn small"
              type="button"
              @click="api.openTarget('taskschd.msc')"
            >
              Open
            </button>
          </li>
        </ul>
      </section>

      <section class="card panel">
        <h2>Maintenance</h2>
        <ul>
          <li>
            <div class="name">
              <strong>Restart Windows Explorer</strong>
              <span class="muted small"
                >Fixes a stuck taskbar or Start menu. Open folders close.</span
              >
            </div>
            <button class="btn" type="button" @click="run('restartExplorer')">Run</button>
          </li>
          <li>
            <div class="name">
              <strong>Empty Recycle Bin</strong>
              <span class="muted small"
                >{{ formatBytes(binBytes) }} — permanent, cannot be undone.</span
              >
            </div>
            <button
              class="btn"
              type="button"
              aria-label="Empty Recycle Bin"
              :disabled="binBytes === 0"
              @click="confirmEmpty = true"
            >
              Empty
            </button>
          </li>
        </ul>

        <h2 class="health-title">Health</h2>
        <ul>
          <li v-for="c in checks" :key="c.id">
            <div class="name">
              <strong>{{ c.title }}</strong>
              <span class="muted small">{{ c.detail }}</span>
            </div>
            <span class="chip" :class="`chip-${c.status}`">{{
              c.status === 'ok' ? 'OK' : c.status === 'warn' ? 'Look at this' : 'Unknown'
            }}</span>
            <button
              v-if="c.status !== 'ok' && c.fixUri"
              class="btn small"
              type="button"
              @click="api.openTarget(c.fixUri)"
            >
              How to fix
            </button>
          </li>
        </ul>
      </section>
    </div>

    <ConfirmDialog
      v-if="confirmEmpty"
      :message="`Permanently delete ${formatBytes(binBytes)} in the Recycle Bin? This cannot be undone.`"
      confirm-label="Empty Recycle Bin"
      danger
      @confirm="run('emptyRecycleBin')"
      @cancel="confirmEmpty = false"
    />
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space) * 2);
}
h1 {
  font-size: 28px;
}
h2 {
  font-size: 17px;
}
.health-title {
  margin-top: calc(var(--space) * 3);
}
.columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: calc(var(--space) * 2);
  align-items: start;
}
.panel {
  padding: calc(var(--space) * 2.5);
}
ul {
  list-style: none;
  padding: 0;
  margin: var(--space) 0 0;
}
li {
  display: flex;
  align-items: center;
  gap: var(--space);
  padding: 10px 0;
  border-bottom: 1px solid var(--border);
}
li:last-child {
  border-bottom: none;
}
.name {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.small {
  font-size: 12px;
}
.status {
  margin: 0;
  color: var(--success);
}
</style>
