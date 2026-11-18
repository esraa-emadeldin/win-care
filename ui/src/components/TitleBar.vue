<script setup lang="ts">
// The window draws its own title bar, like the mockup: a drag area and the
// three Windows buttons. Close hides to the tray (handled in Rust).
import FluentIcon from './FluentIcon.vue'

async function win() {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  return getCurrentWindow()
}
async function minimize() {
  try {
    await (await win()).minimize()
  } catch {
    // Not running inside Tauri (browser preview): nothing to do.
  }
}
async function toggleMaximize() {
  try {
    await (await win()).toggleMaximize()
  } catch {
    // Not running inside Tauri.
  }
}
async function close() {
  try {
    await (await win()).close()
  } catch {
    // Not running inside Tauri.
  }
}
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <button type="button" aria-label="Minimize" @click="minimize">
      <FluentIcon name="minimize" :size="20" />
    </button>
    <button type="button" aria-label="Maximize" @click="toggleMaximize">
      <FluentIcon name="maximize" :size="17" />
    </button>
    <button type="button" class="close" aria-label="Close" @click="close">
      <FluentIcon name="close" :size="20" />
    </button>
  </div>
</template>

<style scoped>
.titlebar {
  position: absolute;
  inset: 0 0 auto 0;
  height: 44px;
  display: flex;
  justify-content: flex-end;
  z-index: 10;
}
button {
  width: 46px;
  height: 40px;
  border: none;
  background: transparent;
  color: var(--text);
  display: grid;
  place-items: center;
}
button:hover {
  background: color-mix(in srgb, var(--text) 8%, transparent);
}
button.close:hover {
  background: #c42b1c;
  color: #fff;
}
</style>
