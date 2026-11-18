<script setup lang="ts">
defineProps<{ disabled?: boolean; danger?: boolean }>()
</script>

<template>
  <button class="primary" :class="{ danger }" type="button" :disabled="disabled">
    <slot />
  </button>
</template>

<style scoped>
.primary {
  background: var(--primary);
  color: #fff;
  border: none;
  border-radius: 999px;
  padding: 12px 36px;
  font-size: 16px;
  font-weight: 600;
  box-shadow: 0 6px 20px color-mix(in srgb, var(--primary) 35%, transparent);
}
.primary:hover:not(:disabled) {
  background: var(--primary-hover);
}
.primary.danger {
  background: var(--error);
  box-shadow: none;
}
.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}
.primary:focus-visible {
  outline: 3px solid var(--accent);
  outline-offset: 2px;
}
</style>
