<script setup lang="ts">
import CardIllustration from './CardIllustration.vue'
import FluentIcon from './FluentIcon.vue'

defineProps<{
  title: string
  /** The big number, e.g. "1.8"; `unit` follows it in a smaller size. */
  value: string
  unit?: string
  caption: string
  art: 'broom' | 'apps' | 'rocket' | 'heart'
  details?: boolean
}>()
defineEmits<{ details: [] }>()
</script>

<template>
  <section class="card scan-card">
    <CardIllustration :name="art" class="art" />
    <h3>{{ title }}</h3>
    <p class="value" :class="{ long: !unit && value.length > 8 }">
      {{ value }}{{ unit ? ' ' : '' }}<span v-if="unit" class="unit">{{ unit }}</span>
    </p>
    <p class="caption">{{ caption }}</p>
    <button v-if="details !== false" class="link" type="button" @click="$emit('details')">
      View details <FluentIcon name="arrowRight" :size="14" />
    </button>
  </section>
</template>

<style scoped>
.scan-card {
  padding: 26px 16px 22px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 6px;
  min-height: 262px;
}
h3 {
  margin-top: 10px;
  font-size: 19px;
  font-weight: 700;
}
.value {
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: 700;
  margin: 2px 0 0;
  line-height: 1.15;
}
/* Phrases like "1 to look at" get a smaller size so they stay on one line. */
.value.long {
  font-size: 24px;
}
.unit {
  font-size: 23px;
  font-weight: 600;
}
.caption {
  margin: 0;
  color: var(--text-2);
  font-size: 15.5px;
}
.link {
  margin-top: auto;
  padding-top: 12px;
  border: none;
  background: none;
  color: var(--primary);
  font-weight: 600;
  font-size: 15.5px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>
