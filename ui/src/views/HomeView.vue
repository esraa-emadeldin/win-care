<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { api, onScanProgress } from '../api'
import { formatBytes } from '../format'
import { startReview, store } from '../stores/scan'
import ScanCard from '../components/ScanCard.vue'
import PrimaryButton from '../components/PrimaryButton.vue'
import FluentIcon from '../components/FluentIcon.vue'

const router = useRouter()
let unlisten: UnlistenFn | undefined

onMounted(async () => {
  unlisten = await onScanProgress((p) => (store.progress = p.label))
})
onUnmounted(() => unlisten?.())

const scan = computed(() => store.scan)
const issues = computed(() => scan.value?.health.filter((c) => c.status === 'warn').length ?? 0)
const cleanable = computed(() => (scan.value?.cleanup.findings ?? []).filter((f) => f.selectable))

/** "1.8 GB" -> ["1.8", "GB"]: the mockup sets the unit smaller. */
function split(text: string): [string, string] {
  const [n = text, u = ''] = text.split(' ')
  return [n, u]
}
const junk = computed(() => (scan.value ? split(formatBytes(scan.value.cleanupBytes)) : ['—', '']))

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
      <p class="subtitle">Let's check your PC and find what can be cleaned up.</p>
    </header>

    <div class="cards">
      <ScanCard
        data-testid="card-cleanup"
        art="broom"
        title="Cleanup"
        :value="junk[0]"
        :unit="junk[1]"
        caption="Junk files"
        @details="clean"
      />
      <ScanCard
        data-testid="card-apps"
        art="apps"
        title="Apps"
        :value="scan ? String(scan.appsCount) : '—'"
        :unit="scan ? 'apps' : ''"
        :caption="scan ? `${formatBytes(scan.appsBytes)} installed` : 'Installed apps'"
        @details="router.push('/apps')"
      />
      <ScanCard
        data-testid="card-startup"
        art="rocket"
        title="Startup"
        :value="scan ? String(scan.startupCount) : '—'"
        :unit="scan ? 'items' : ''"
        caption="start with Windows"
        @details="router.push('/performance')"
      />
      <ScanCard
        data-testid="card-health"
        art="heart"
        title="Health"
        :value="scan ? (issues ? `${issues} to look at` : 'Looks good!') : '—'"
        :caption="scan ? (issues ? 'See Performance' : 'No issues found') : 'Security and disks'"
        :details="scan !== null && issues > 0"
        @details="router.push('/performance')"
      />
    </div>

    <ul v-if="scan && (scan.cleanup.notes.length || scan.cleanup.unreadableDirs)" class="notes">
      <li v-for="n in scan.cleanup.notes" :key="n">{{ n }}</li>
      <li v-if="scan.cleanup.unreadableDirs">
        {{ scan.cleanup.unreadableDirs }} folders could not be read and were skipped
      </li>
    </ul>

    <div class="actions">
      <PrimaryButton v-if="!scan" :disabled="store.scanning" @click="runScan">
        <FluentIcon name="wand" :size="24" /> Scan
      </PrimaryButton>
      <template v-else>
        <PrimaryButton :disabled="store.scanning || cleanable.length === 0" @click="clean">
          <FluentIcon name="wand" :size="24" /> Clean
        </PrimaryButton>
        <button class="again" type="button" :disabled="store.scanning" @click="runScan">
          Scan again
        </button>
      </template>
      <p class="progress" aria-live="polite">{{ store.progress }}</p>
    </div>
  </div>
</template>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  gap: 22px;
  max-width: 1040px;
}
.cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}
.notes {
  list-style: none;
  padding: 0;
  margin: -6px 0 0;
  color: var(--text-3);
  font-size: var(--fs-small);
  text-align: center;
}
.actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.again {
  border: none;
  background: none;
  color: var(--primary);
  font-weight: 600;
}
.progress {
  margin: 0;
  min-height: 1.2em;
  color: var(--text-3);
}
</style>
