<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api'
import { formatBytes } from '../format'
import { startReview } from '../stores/scan'
import type { App } from '../types'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import AppAvatar from '../components/AppAvatar.vue'
import FluentIcon from '../components/FluentIcon.vue'

const router = useRouter()
const apps = ref<App[]>([])
const query = ref('')
const sort = ref<'size' | 'name'>('size')
const confirming = ref<App | null>(null)
const busy = ref('')
const message = ref('')
const menuFor = ref<string | null>(null)

/** Install folder recorded by the installer, if any. */
function installFolder(app: App): string | null {
  const kind = app.kind as { install_location?: string | null } | null
  return kind?.install_location ?? null
}

async function openFolder(app: App) {
  menuFor.value = null
  const folder = installFolder(app)
  if (folder) await api.revealPath(folder)
}

onMounted(async () => {
  apps.value = await api.listApps()
})

const shown = computed(() => {
  const q = query.value.trim().toLowerCase()
  const list = apps.value.filter(
    (a) => !q || a.name.toLowerCase().includes(q) || (a.publisher ?? '').toLowerCase().includes(q)
  )
  return sort.value === 'size'
    ? [...list].sort((a, b) => (b.bytes ?? -1) - (a.bytes ?? -1))
    : [...list].sort((a, b) => a.name.localeCompare(b.name))
})

async function uninstall(app: App) {
  confirming.value = null
  busy.value = app.id
  message.value = ''
  try {
    const leftovers = await api.uninstallApp(app.id)
    apps.value = await api.listApps()
    if (leftovers.length) {
      startReview('leftovers', leftovers)
      router.push('/review')
    } else {
      message.value = apps.value.some((a) => a.id === app.id)
        ? `${app.name} is still installed.`
        : `${app.name} was removed. Nothing was left behind.`
    }
  } catch (e) {
    message.value = String(e)
  } finally {
    busy.value = ''
  }
}
</script>

<template>
  <div class="page">
    <div class="page-head">
      <div>
        <h1>Applications</h1>
        <p class="subtitle">Manage your installed apps and remove unwanted ones.</p>
      </div>
      <div class="tools">
        <label class="search field">
          <FluentIcon name="search" :size="18" />
          <input
            v-model="query"
            type="search"
            placeholder="Search apps..."
            aria-label="Search apps"
          />
        </label>
        <label class="sort field">
          <select v-model="sort" aria-label="Sort by">
            <option value="size">Sort by: Size</option>
            <option value="name">Sort by: Name</option>
          </select>
          <FluentIcon name="chevronDown" :size="16" />
        </label>
      </div>
    </div>

    <p v-if="message" class="status" role="status">{{ message }}</p>

    <div class="card list">
      <table>
        <tbody>
          <tr v-for="app in shown" :key="app.id">
            <td class="name">
              <AppAvatar :name="app.name" :size="40" />
              <span>
                <strong>{{ app.name }}</strong>
                <span class="muted">{{ app.publisher ?? '' }}</span>
              </span>
            </td>
            <td class="size">{{ app.bytes ? formatBytes(app.bytes) : '—' }}</td>
            <td class="used">Last used: unknown</td>
            <td class="action">
              <button class="btn" type="button" :disabled="busy !== ''" @click="confirming = app">
                {{ busy === app.id ? 'Uninstalling…' : 'Uninstall' }}
              </button>
              <span class="menu-wrap">
                <button
                  class="more"
                  type="button"
                  :aria-label="`More for ${app.name}`"
                  :aria-expanded="menuFor === app.id"
                  @click="menuFor = menuFor === app.id ? null : app.id"
                >
                  <FluentIcon name="more" :size="20" />
                </button>
                <div v-if="menuFor === app.id" class="card menu" role="menu">
                  <button
                    type="button"
                    role="menuitem"
                    :disabled="!installFolder(app)"
                    @click="openFolder(app)"
                  >
                    Open install folder
                  </button>
                </div>
              </span>
            </td>
          </tr>
        </tbody>
      </table>
      <p class="info">
        <span class="shield"><FluentIcon name="shield" :size="20" /></span>
        After uninstalling, win-care shows any leftover files for you to review.
      </p>
    </div>

    <ConfirmDialog
      v-if="confirming"
      :message="`Uninstall ${confirming.name}? Its own uninstaller will open.`"
      confirm-label="Uninstall"
      @confirm="uninstall(confirming!)"
      @cancel="confirming = null"
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
.tools {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
}
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 340px;
  color: var(--text-3);
}
.search input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font: inherit;
  color: var(--text);
}
.sort {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
}
.sort select {
  appearance: none;
  border: none;
  background: transparent;
  font: inherit;
  color: var(--text);
  padding-right: 4px;
  cursor: pointer;
}
.status {
  margin: 0;
  color: var(--text-2);
}
.list {
  padding: 6px 14px 14px;
}
table {
  border-collapse: collapse;
  width: 100%;
}
td {
  padding: 12px 10px;
  border-bottom: 1px solid var(--divider);
  white-space: nowrap;
}
.name {
  display: flex;
  align-items: center;
  gap: 16px;
  white-space: normal;
}
.name > span {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name strong {
  font-size: 17px;
}
.size {
  font-size: 16px;
  font-weight: 600;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.used {
  color: var(--text);
  font-size: 16px;
  padding-left: 40px;
}
.action {
  text-align: right;
}
.menu-wrap {
  position: relative;
  display: inline-block;
}
.more {
  border: none;
  background: none;
  color: var(--text-2);
  padding: 6px;
  margin-left: 6px;
  border-radius: 8px;
  vertical-align: middle;
}
.more:hover {
  background: var(--glass-strong);
}
.menu {
  position: absolute;
  right: 0;
  top: 100%;
  z-index: 5;
  padding: 6px;
  min-width: 190px;
  background: var(--glass-strong);
}
.menu button {
  width: 100%;
  text-align: left;
  border: none;
  background: none;
  color: var(--text);
  padding: 8px 10px;
  border-radius: 8px;
}
.menu button:hover:not(:disabled) {
  background: var(--glass-hover);
}
.menu button:disabled {
  color: var(--text-3);
  cursor: not-allowed;
}
.info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 14px 0 0;
  padding: 12px 16px;
  border-radius: 12px;
  background: color-mix(in srgb, var(--primary) 7%, var(--glass-strong));
  font-weight: 500;
}
.shield {
  color: var(--primary);
  display: inline-flex;
}
</style>
