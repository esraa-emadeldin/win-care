<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api'
import { formatBytes } from '../format'
import {
  CATEGORY_DESCRIPTIONS,
  CATEGORY_LABELS,
  type ReviewItem,
  baseName,
  reviewItems,
  selectedTotal,
  store
} from '../stores/scan'
import type { ApplyReport, CategoryId, SkipReason } from '../types'
import ReviewGroup from '../components/ReviewGroup.vue'
import PrimaryButton from '../components/PrimaryButton.vue'
import FluentIcon from '../components/FluentIcon.vue'

const router = useRouter()
const items = ref<ReviewItem[]>([])
const loading = ref(true)
const busy = ref(false)
const error = ref('')
const report = ref<ApplyReport | null>(null)

onMounted(async () => {
  const findings = store.review.findings
  try {
    // Ask the backend which items the Recycle Bin can take, before anything is picked.
    const plan = await api.makePlan(findings.filter((f) => f.selectable).map((f) => f.path))
    const unrecyclable = new Set(plan.items.filter((i) => !i.recyclable).map((i) => i.finding.path))
    const blocked = new Map(plan.rejected.map((r) => [r.path, reasonText(r.reason)]))
    items.value = reviewItems(findings, unrecyclable, blocked)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

const groups = computed(() => {
  const by = new Map<CategoryId, ReviewItem[]>()
  for (const i of items.value)
    by.set(i.finding.category, [...(by.get(i.finding.category) ?? []), i])
  return [...by.entries()].map(([cat, list]) => ({
    title: CATEGORY_LABELS[cat],
    description: CATEGORY_DESCRIPTIONS[cat],
    items: list
  }))
})
const total = computed(() => selectedTotal(items.value))
const pickable = computed(() =>
  items.value.filter((i) => i.finding.selectable && i.recyclable && i.blocked === null)
)
const allSelected = computed(
  () => pickable.value.length > 0 && pickable.value.every((i) => i.selected)
)
function selectAll(e: Event) {
  const on = (e.target as HTMLInputElement).checked
  for (const i of pickable.value) i.selected = on
}
const selected = computed(() =>
  items.value.filter(
    (i) => i.selected && i.finding.selectable && i.recyclable && i.blocked === null
  )
)

async function moveToBin() {
  busy.value = true
  error.value = ''
  try {
    const plan = await api.makePlan(
      selected.value.map((i) => i.finding.path),
      'recycle'
    )
    report.value = await api.applyPlan(plan.id)
  } catch (e) {
    error.value = String(e)
  } finally {
    busy.value = false
  }
}

const REASONS: Record<string, string> = {
  inUse: 'in use',
  changed: 'changed since the scan',
  missing: 'already gone',
  notRecyclable: 'too large for the Recycle Bin',
  protected: 'protected folder',
  needsAdmin: 'needs admin'
}

function reasonText(r: SkipReason): string {
  return typeof r === 'string' ? (REASONS[r] ?? r) : `failed: ${r.failed}`
}

function done() {
  store.scan = null
  router.push('/')
}
</script>

<template>
  <div class="review">
    <div class="page-head">
      <div>
        <h1>Review Before Deleting</h1>
        <p class="subtitle">
          Please review the items below. Nothing will be deleted until you confirm.
        </p>
      </div>
      <label v-if="!report && pickable.length" class="select-all">
        <input type="checkbox" :checked="allSelected" @change="selectAll" /> Select All
      </label>
    </div>

    <p v-if="loading" class="muted">Checking each item…</p>
    <p v-if="error" class="error" role="alert">{{ error }}</p>

    <template v-if="!report && !loading">
      <div class="groups">
        <ReviewGroup
          v-for="g in groups"
          :key="g.title"
          :title="g.title"
          :description="g.description"
          :items="g.items"
        />
      </div>
      <p v-if="items.length === 0" class="muted">Nothing to clean right now.</p>
      <footer>
        <button class="btn" type="button" @click="router.back()">Back</button>
        <button
          class="btn btn-primary move"
          type="button"
          :disabled="busy || total === 0"
          @click="moveToBin"
        >
          <FluentIcon name="trash" :size="20" />
          Move to Recycle Bin ({{ formatBytes(total) }})
        </button>
      </footer>
    </template>

    <section v-if="report" class="card result" aria-live="polite">
      <h2>Moved {{ report.done.length }} items, freed {{ formatBytes(report.freedBytes) }}</h2>
      <p class="muted">You can restore anything from the Recycle Bin.</p>
      <template v-if="report.skipped.length">
        <h3>Skipped</h3>
        <ul>
          <li v-for="s in report.skipped" :key="s.path">
            <strong>{{ baseName(s.path) }}</strong>
            <span class="muted"> — {{ reasonText(s.reason) }}</span>
          </li>
        </ul>
      </template>
      <div class="result-actions">
        <button class="btn" type="button" @click="api.openTarget('shell:RecycleBinFolder')">
          Open Recycle Bin
        </button>
        <PrimaryButton @click="done">Done</PrimaryButton>
      </div>
    </section>
  </div>
</template>

<style scoped>
.review {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1040px;
}
.select-all {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15.5px;
  font-weight: 500;
  cursor: pointer;
}
.groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
footer,
.result-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
}
footer .btn:not(.btn-primary) {
  padding: 11px 30px;
  font-size: 15.5px;
}
.move {
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 12px;
}
.result {
  padding: 26px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.result h2 {
  font-size: 22px;
}
.error {
  color: var(--error);
}
</style>
