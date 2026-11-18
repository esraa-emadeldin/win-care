<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { api } from '../api'
import type { IconName } from '../icons'
import FluentIcon from '../components/FluentIcon.vue'
import TitleBar from '../components/TitleBar.vue'

const route = useRoute()
const router = useRouter()
let unlistenNav: UnlistenFn | undefined

// The tray popup asks the main window to open a page (Settings, Smart Scan).
onMounted(async () => {
  try {
    unlistenNav = await listen<string>('navigate', (e) => router.push(e.payload))
  } catch {
    // Not inside Tauri.
  }
})
onUnmounted(() => unlistenNav?.())
const content = ref<HTMLElement>()

// One scroll area serves every page: start each page at the top.
watch(
  () => route.path,
  () => content.value?.scrollTo({ top: 0 })
)
const update = ref<string | null>(null)
const installing = ref(false)

// Only checks when the user turned it on, at most once a day (decided in Rust).
onMounted(async () => {
  try {
    update.value = await api.checkUpdate()
  } catch {
    // Offline or no release yet: say nothing.
  }
})

async function install() {
  installing.value = true
  try {
    await api.installUpdate()
  } finally {
    installing.value = false
  }
}

const links: { to: string; label: string; icon: IconName; active: IconName }[] = [
  { to: '/', label: 'Home', icon: 'home', active: 'homeFilled' },
  { to: '/apps', label: 'Applications', icon: 'apps', active: 'appsFilled' },
  { to: '/performance', label: 'Performance', icon: 'gauge', active: 'gaugeFilled' },
  { to: '/space', label: 'Space Map', icon: 'treemap', active: 'treemapFilled' },
  { to: '/settings', label: 'Settings', icon: 'settings', active: 'settingsFilled' }
]

// Review belongs to Home in the menu.
const isActive = (to: string) =>
  to === '/' ? route.path === '/' || route.path === '/review' : route.path === to
</script>

<template>
  <div class="shell">
    <svg class="waves" viewBox="0 0 1200 800" preserveAspectRatio="none" aria-hidden="true">
      <path d="M0 650 C300 560 520 760 820 640 C1000 570 1120 600 1200 560 V800 H0 Z" />
      <path d="M600 0 C760 120 900 60 1200 180 V0 Z" />
    </svg>

    <nav aria-label="Main">
      <div class="brand">
        <span class="brand-mark"><FluentIcon name="shield" :size="30" /></span>
        <span>win-care</span>
      </div>
      <RouterLink
        v-for="l in links"
        :key="l.to"
        :to="l.to"
        class="nav-link"
        :class="{ active: isActive(l.to) }"
      >
        <FluentIcon :name="isActive(l.to) ? l.active : l.icon" :size="24" />
        {{ l.label }}
      </RouterLink>
      <RouterLink to="/about" class="nav-link about" :class="{ active: isActive('/about') }">
        <FluentIcon :name="isActive('/about') ? 'infoFilled' : 'info'" :size="22" />
        About
      </RouterLink>
    </nav>

    <main>
      <TitleBar />
      <div ref="content" class="content">
        <div v-if="update" class="card update" role="status">
          <span>win-care {{ update }} is available.</span>
          <button class="btn btn-primary" type="button" :disabled="installing" @click="install">
            {{ installing ? 'Installing…' : 'Install' }}
          </button>
          <button class="btn" type="button" @click="update = null">Not now</button>
        </div>
        <RouterView />
      </div>
    </main>
  </div>
</template>

<style scoped>
.shell {
  position: relative;
  display: grid;
  grid-template-columns: 248px 1fr;
  grid-template-rows: 100%;
  height: 100%;
  overflow: hidden;
}
/* Soft light waves behind everything, as in the mockup's backdrop. */
.waves {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  fill: var(--wave);
  pointer-events: none;
}
nav {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 22px 14px 18px;
  background: var(--sidebar);
  border-right: 1px solid var(--border);
  backdrop-filter: blur(20px);
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--font-display);
  font-weight: 700;
  font-size: 22px;
  padding: 2px 10px 30px;
}
.brand-mark {
  color: var(--primary);
  display: inline-flex;
}
.nav-link {
  position: relative;
  display: flex;
  gap: 14px;
  align-items: center;
  padding: 13px 16px;
  border-radius: 12px;
  color: var(--text);
  text-decoration: none;
  font-size: 16.5px;
}
.nav-link:hover {
  background: color-mix(in srgb, var(--glass-strong) 60%, transparent);
}
.nav-link.active {
  background: color-mix(in srgb, var(--primary) 9%, var(--glass-strong));
  box-shadow: var(--shadow);
  color: var(--primary);
  font-weight: 600;
}
/* The blue accent bar on the active item. */
.nav-link.active::before {
  content: '';
  position: absolute;
  left: -2px;
  top: 10px;
  bottom: 10px;
  width: 3px;
  border-radius: 3px;
  background: var(--primary);
}
.about {
  margin-top: auto;
}
main {
  position: relative;
  min-width: 0;
  min-height: 0;
  height: 100%;
}
.content {
  position: relative;
  height: 100%;
  overflow: auto;
  padding: 30px 36px 28px;
}
.update {
  display: flex;
  align-items: center;
  gap: var(--space);
  padding: 10px 16px;
  margin-bottom: calc(var(--space) * 2);
}
.update span {
  flex: 1;
}
</style>
