import { createApp } from 'vue'
import App from './App.vue'
import { router } from './router'
import { api } from './api'
import { applyTheme } from './theme'
import { hideSplash, removeSplashNow } from './splash'
import './styles/tokens.css'

async function start() {
  if (location.hash.startsWith('#/tray')) removeSplashNow()
  // `vite --mode mock`: a fake backend, for tests and UI work without Rust.
  if (import.meta.env.MODE === 'mock') {
    const { installMock } = await import('./mock')
    installMock()
  }
  try {
    applyTheme((await api.getSettings()).theme)
  } catch {
    // Settings unavailable: keep the system theme.
  }
  createApp(App).use(router).mount('#app')
  // Wait for the first screen to be ready, then reveal it.
  await router.isReady()
  hideSplash()
}

start()
