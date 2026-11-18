<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'

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

const links = [
  { to: '/', label: 'Home', icon: '🏠' },
  { to: '/apps', label: 'Applications', icon: '🧩' },
  { to: '/performance', label: 'Performance', icon: '⚡' },
  { to: '/space', label: 'Space Map', icon: '🗺️' },
  { to: '/settings', label: 'Settings', icon: '⚙️' }
]
</script>

<template>
  <div class="shell">
    <nav aria-label="Main">
      <div class="brand">
        <img src="/icon.svg" alt="" width="28" height="28" />
        <span>win-care</span>
      </div>
      <RouterLink v-for="l in links" :key="l.to" :to="l.to" class="nav-link">
        <span aria-hidden="true">{{ l.icon }}</span> {{ l.label }}
      </RouterLink>
      <RouterLink to="/about" class="nav-link about">
        <span aria-hidden="true">ℹ️</span> About
      </RouterLink>
    </nav>
    <main>
      <div v-if="update" class="card update" role="status">
        <span>win-care {{ update }} is available.</span>
        <button class="btn" type="button" :disabled="installing" @click="install">
          {{ installing ? 'Installing…' : 'Install' }}
        </button>
        <button class="btn" type="button" @click="update = null">Not now</button>
      </div>
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
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
.shell {
  display: grid;
  grid-template-columns: 220px 1fr;
  height: 100%;
}
nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: calc(var(--space) * 2);
}
.brand {
  display: flex;
  align-items: center;
  gap: var(--space);
  font-family: var(--font-display);
  font-weight: 700;
  font-size: 17px;
  padding: var(--space) var(--space) calc(var(--space) * 3);
}
.nav-link {
  display: flex;
  gap: var(--space);
  align-items: center;
  padding: 8px 12px;
  border-radius: 8px;
  color: var(--text);
  text-decoration: none;
}
.nav-link:hover {
  background: color-mix(in srgb, var(--text) 6%, transparent);
}
.nav-link.router-link-exact-active {
  background: var(--surface);
  box-shadow: var(--shadow);
  color: var(--primary);
  font-weight: 600;
}
.about {
  margin-top: auto;
}
main {
  overflow: auto;
  padding: calc(var(--space) * 4);
  background: var(--surface-alt);
  border-top-left-radius: var(--radius);
}
</style>
