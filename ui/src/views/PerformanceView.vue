<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api } from '../api'
import { formatBytes } from '../format'
import type { Check, StartupItem } from '../types'
import ToggleSwitch from '../components/ToggleSwitch.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import AppAvatar from '../components/AppAvatar.vue'
import FluentIcon from '../components/FluentIcon.vue'

type Task = 'restartExplorer' | 'emptyRecycleBin'
/** Startup rows shown before "See more apps". */
const FIRST_ROWS = 4

const startup = ref<StartupItem[]>([])
const checks = ref<Check[]>([])
const binBytes = ref(0)
const showAll = ref(false)
// Nothing is ticked by default: one task closes Explorer windows, the other
// deletes permanently. The user chooses.
const picked = ref<Record<Task, boolean>>({ restartExplorer: false, emptyRecycleBin: false })
const confirmEmpty = ref(false)
const running = ref(false)
const message = ref('')

onMounted(async () => {
  ;[startup.value, checks.value, binBytes.value] = await Promise.all([
    api.listStartup(),
    api.health(),
    api.recycleBinSize()
  ])
})

const shownStartup = computed(() =>
  showAll.value ? startup.value : startup.value.slice(0, FIRST_ROWS)
)
const anyPicked = computed(() => picked.value.restartExplorer || picked.value.emptyRecycleBin)

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

/** Emptying the Recycle Bin is permanent, so it always asks first. */
function runMaintenance() {
  if (picked.value.emptyRecycleBin) confirmEmpty.value = true
  else run()
}

async function run() {
  confirmEmpty.value = false
  running.value = true
  const done: string[] = []
  try {
    for (const task of ['restartExplorer', 'emptyRecycleBin'] as Task[]) {
      if (picked.value[task]) done.push(await api.runMaintenance(task))
    }
    binBytes.value = await api.recycleBinSize()
    picked.value = { restartExplorer: false, emptyRecycleBin: false }
    message.value = done.join(' · ')
  } catch (e) {
    message.value = String(e)
  } finally {
    running.value = false
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h1>Performance</h1>
        <p class="subtitle">Keep your PC running smoothly.</p>
      </div>
      <button
        class="btn btn-primary run"
        type="button"
        :disabled="!anyPicked || running"
        @click="runMaintenance"
      >
        <FluentIcon name="play" :size="18" /> Run Maintenance
      </button>
    </div>
    <p v-if="message" role="status" class="status">{{ message }}</p>

    <div class="columns">
      <section class="card panel">
        <h2>Startup Apps</h2>
        <p class="panel-sub">Control what runs at startup.</p>
        <ul>
          <li v-for="item in shownStartup" :key="item.id">
            <AppAvatar :name="item.name" :size="26" />
            <div class="name">
              <span>{{ item.name }}</span>
              <span v-if="!item.canToggle" class="note">{{ readOnlyNote(item) }}</span>
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
        <button
          v-if="startup.length > FIRST_ROWS"
          class="link"
          type="button"
          @click="showAll = !showAll"
        >
          {{ showAll ? 'See fewer apps' : 'See more apps' }}
          <FluentIcon name="chevronDown" :size="16" :class="{ flip: showAll }" />
        </button>
      </section>

      <section class="card panel">
        <h2 class="tasks-title">Maintenance Tasks</h2>
        <ul class="tasks">
          <li>
            <label>
              <input v-model="picked.restartExplorer" type="checkbox" />
              <span class="name">
                <span>Restart Windows Explorer</span>
                <span class="note">Fixes a stuck taskbar or Start menu. Open folders close.</span>
              </span>
            </label>
          </li>
          <li>
            <label>
              <input
                v-model="picked.emptyRecycleBin"
                type="checkbox"
                :disabled="binBytes === 0"
                aria-label="Empty Recycle Bin"
              />
              <span class="name">
                <span>Empty Recycle Bin</span>
                <span class="note">{{ formatBytes(binBytes) }} — permanent, cannot be undone.</span>
              </span>
            </label>
          </li>
        </ul>
      </section>

      <section class="card panel health">
        <h2>Health</h2>
        <ul>
          <li v-for="c in checks" :key="c.id">
            <div class="name">
              <span>{{ c.title }}</span>
              <span class="note">{{ c.detail }}</span>
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
      @confirm="run"
      @cancel="confirmEmpty = false"
    />
  </div>
</template>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1040px;
}
.run {
  padding: 11px 22px;
  border-radius: 12px;
  font-size: 15.5px;
}
.status {
  margin: 0;
  color: var(--success);
  font-weight: 600;
}
.columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
  align-items: start;
}
.panel {
  padding: 22px 24px;
}
.health {
  grid-column: span 2;
}
h2 {
  font-size: 21px;
  font-weight: 700;
}
.tasks-title {
  font-size: 18px;
}
.panel-sub {
  margin: 4px 0 6px;
  color: var(--text-2);
  font-size: 15px;
}
ul {
  list-style: none;
  padding: 0;
  margin: 8px 0 0;
}
li {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 11px 0;
  border-bottom: 1px solid var(--divider);
}
li:last-child {
  border-bottom: none;
}
.tasks li {
  border-bottom: none;
  padding: 8px 0;
}
label {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  cursor: pointer;
}
label input {
  margin-top: 3px;
}
.name {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  font-size: 16px;
  font-weight: 500;
}
.note {
  font-size: 14px;
  font-weight: 400;
  color: var(--text-2);
}
.small {
  padding: 5px 12px;
  font-size: 13px;
}
.link {
  margin-top: 10px;
  border: none;
  background: none;
  color: var(--primary);
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
}
.flip {
  transform: rotate(180deg);
}
</style>
