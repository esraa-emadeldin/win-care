<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
const model = defineModel<boolean>({ required: true })
</script>

<template>
  <button
    class="toggle"
    type="button"
    role="switch"
    :aria-label="label"
    :aria-checked="model"
    :disabled="disabled"
    @click="model = !model"
  >
    <span class="knob" />
  </button>
</template>

<style scoped>
.toggle {
  width: 40px;
  height: 22px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface-alt);
  padding: 2px;
  position: relative;
  flex: none;
}
.toggle[aria-checked='true'] {
  background: var(--primary);
  border-color: var(--primary);
}
.knob {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--text-2);
  transition: transform 0.15s ease;
}
.toggle[aria-checked='true'] .knob {
  background: #fff;
  transform: translateX(18px);
}
.toggle:disabled {
  opacity: 0.5;
}
@media (prefers-reduced-motion: reduce) {
  .knob {
    transition: none;
  }
}
</style>
