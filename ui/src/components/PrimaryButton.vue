<script setup lang="ts">
defineProps<{ disabled?: boolean; danger?: boolean }>()
</script>

<template>
  <button class="primary" :class="{ danger }" type="button" :disabled="disabled">
    <slot />
  </button>
</template>

<style scoped>
/* The mockup's big "Clean" pill: blue gradient, soft glow. */
.primary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-width: 220px;
  height: 60px;
  padding: 0 40px;
  background: var(--primary-gradient);
  color: #fff;
  border: none;
  border-radius: 999px;
  font-family: var(--font-display);
  font-size: 21px;
  font-weight: 600;
  box-shadow: var(--shadow-strong);
}
.primary:hover:not(:disabled) {
  filter: brightness(1.08);
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
  outline-offset: 3px;
}
</style>
