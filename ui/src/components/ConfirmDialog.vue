<script setup lang="ts">
import { onMounted, ref } from 'vue'

defineProps<{ message: string; confirmLabel: string; danger?: boolean }>()
const emit = defineEmits<{ confirm: []; cancel: [] }>()
const dialog = ref<HTMLDialogElement>()

// A native modal dialog: traps focus and closes on Esc by itself.
onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog ref="dialog" class="card" @cancel.prevent="emit('cancel')">
    <p>{{ message }}</p>
    <div class="buttons">
      <button class="btn" type="button" autofocus @click="emit('cancel')">Cancel</button>
      <button class="btn confirm" :class="{ danger }" type="button" @click="emit('confirm')">
        {{ confirmLabel }}
      </button>
    </div>
  </dialog>
</template>

<style scoped>
dialog {
  padding: calc(var(--space) * 3);
  max-width: 440px;
  color: var(--text);
}
dialog::backdrop {
  background: rgb(15 23 42 / 0.35);
}
p {
  margin: 0 0 calc(var(--space) * 3);
  font-size: 15px;
}
.buttons {
  display: flex;
  justify-content: flex-end;
  gap: var(--space);
}
.confirm {
  background: var(--primary);
  border-color: var(--primary);
  color: #fff;
}
.confirm.danger {
  background: var(--error);
  border-color: var(--error);
}
</style>
