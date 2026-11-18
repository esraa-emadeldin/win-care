<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api'
import { formatBytes } from '../format'
import { startReview } from '../stores/scan'
import type { App } from '../types'
import ConfirmDialog from '../components/ConfirmDialog.vue'

const router = useRouter()
const apps = ref<App[]>([])
const query = ref('')
const sort = ref<'size' | 'name'>('size')
const confirming = ref<App | null>(null)
const busy = ref('')
const message = ref('')

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
    <header>
      <div>
        <h1>Applications</h1>
        <p class="muted">Manage your installed apps and remove unwanted ones.</p>
      </div>
      <div class="tools">
        <input v-model="query" type="search" placeholder="Search apps…" aria-label="Search apps" />
        <select v-model="sort" aria-label="Sort by">
          <option value="size">Sort by: Size</option>
          <option value="name">Sort by: Name</option>
        </select>
      </div>
    </header>

    <p v-if="message" class="muted" role="status">{{ message }}</p>

    <table class="card">
      <tbody>
        <tr v-for="app in shown" :key="app.id">
          <td class="name">
            <strong>{{ app.name }}</strong>
            <span class="muted">{{ app.publisher ?? '' }}</span>
          </td>
          <td class="size">{{ app.bytes ? formatBytes(app.bytes) : '—' }}</td>
          <td class="muted">Last used: unknown</td>
          <td class="action">
            <button class="btn" type="button" :disabled="busy !== ''" @click="confirming = app">
              {{ busy === app.id ? 'Uninstalling…' : 'Uninstall' }}
            </button>
          </td>
        </tr>
      </tbody>
    </table>
    <p class="muted note">
      🛡️ After an app's own uninstaller finishes, win-care shows you what it left behind.
    </p>

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
  gap: calc(var(--space) * 2);
}
header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: var(--space);
}
h1 {
  font-size: 28px;
}
.tools {
  display: flex;
  gap: var(--space);
}
input,
select {
  font: inherit;
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text);
}
table {
  border-collapse: collapse;
  width: 100%;
  overflow: hidden;
}
td {
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
}
tr:last-child td {
  border-bottom: none;
}
.name {
  display: flex;
  flex-direction: column;
}
.size {
  font-variant-numeric: tabular-nums;
  text-align: right;
}
.action {
  text-align: right;
}
.note {
  margin: 0;
}
</style>
