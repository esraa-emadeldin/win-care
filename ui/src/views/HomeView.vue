<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { api, onScanProgress } from '../api'
import { formatBytes } from '../format'
import { startReview, store } from '../stores/scan'
import ScanCard from '../components/ScanCard.vue'
import PrimaryButton from '../components/PrimaryButton.vue'

const router = useRouter()
let unlisten: UnlistenFn | undefined

onMounted(async () => {
  unlisten = await onScanProgress((p) => (store.progress = p.label))
})
onUnmounted(() => unlisten?.())

const scan = computed(() => store.scan)
const issues = computed(() => scan.value?.health.filter((c) => c.status === 'warn').length ?? 0)
const cleanable = computed(() => (scan.value?.cleanup.findings ?? []).filter((f) => f.selectable))

async function runScan() {
  store.scanning = true
  store.progress = 'Starting…'
  try {
    store.scan = await api.smartScan()
  } finally {
    store.scanning = false
    store.progress = ''
  }
}

function clean() {
  startReview('cleanup', cleanable.value)
  router.push('/review')
}
</script>

<template>
  <div class="home">
    <header>
      <h1>Smart Scan</h1>
      <p class="muted">Let's check your PC and find what can be cleaned up.</p>
    </header>

    <div class="cards">
      <ScanCard
        data-testid="card-cleanup"
        icon="🧹"
        title="Cleanup"
        :value="scan ? formatBytes(scan.cleanupBytes) : '—'"
        caption="Junk files"
        @details="clean"
      />
      <ScanCard
        data-testid="card-apps"
        icon="🧩"
        title="Apps"
        :value="scan ? `${scan.appsCount} apps` : '—'"
        :caption="scan ? `${formatBytes(scan.appsBytes)} installed` : 'Installed apps'"
        @details="router.push('/apps')"
      />
      <ScanCard
        data-testid="card-startup"
        icon="🚀"
        title="Startup"
        :value="scan ? `${scan.startupCount} items` : '—'"
        caption="start with Windows"
        @details="router.push('/performance')"
      />
      <ScanCard
        data-testid="card-health"
        icon="💚"
        title="Health"
        :tone="scan ? (issues ? 'warn' : 'ok') : undefined"
        :value="scan ? (issues ? `${issues} to look at` : 'Looks good!') : '—'"
        :caption="scan ? (issues ? 'See Performance' : 'No issues found') : 'Security and disks'"
        @details="router.push('/performance')"
      />
    </div>

    <ul v-if="scan && (scan.cleanup.notes.length || scan.cleanup.unreadableDirs)" class="notes">
      <li v-for="n in scan.cleanup.notes" :key="n" class="muted">ℹ️ {{ n }}</li>
      <li v-if="scan.cleanup.unreadableDirs" class="muted">
        ℹ️ {{ scan.cleanup.unreadableDirs }} folders could not be read and were skipped
      </li>
    </ul>

    <div class="actions">
      <PrimaryButton v-if="!scan" :disabled="store.scanning" @click="runScan">Scan</PrimaryButton>
      <template v-else>
        <PrimaryButton :disabled="store.scanning || cleanable.length === 0" @click="clean">
          Clean
        </PrimaryButton>
        <button class="btn" type="button" :disabled="store.scanning" @click="runScan">
          Scan again
        </button>
      </template>
      <p class="muted" aria-live="polite">{{ store.progress }}</p>
    </div>
  </div>
</template>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space) * 3);
}
h1 {
  font-size: 28px;
}
.cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: calc(var(--space) * 2);
}
.notes {
  list-style: none;
  padding: 0;
  margin: 0;
}
.actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space);
}
</style>
