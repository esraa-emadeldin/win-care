<script setup lang="ts">
defineProps<{
  title: string
  value: string
  caption: string
  icon: string
  tone?: 'ok' | 'warn'
}>()
defineEmits<{ details: [] }>()
</script>

<template>
  <section class="card scan-card">
    <div class="icon" :class="tone" aria-hidden="true">{{ icon }}</div>
    <h3>{{ title }}</h3>
    <p class="value">{{ value }}</p>
    <p class="muted caption">{{ caption }}</p>
    <button class="link" type="button" @click="$emit('details')">View details →</button>
  </section>
</template>

<style scoped>
.scan-card {
  padding: calc(var(--space) * 3);
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: calc(var(--space) / 2);
}
.icon {
  font-size: 28px;
  width: 52px;
  height: 52px;
  display: grid;
  place-items: center;
  border-radius: 14px;
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  margin-bottom: var(--space);
}
.icon.ok {
  background: color-mix(in srgb, var(--success) 14%, transparent);
}
.icon.warn {
  background: color-mix(in srgb, var(--warning) 18%, transparent);
}
h3 {
  font-size: 15px;
  font-weight: 600;
}
.value {
  font-family: var(--font-display);
  font-size: 26px;
  font-weight: 600;
  margin: 0;
}
.caption {
  margin: 0 0 var(--space);
  min-height: 1.4em;
}
.link {
  border: none;
  background: none;
  color: var(--primary);
  font-weight: 600;
}
</style>
