import { defineConfig } from '@playwright/test'

// UI tests run against the built UI with the Tauri backend mocked (VITE_MOCK=1),
// so they need no Rust build and touch nothing on this PC.
const isCI = !!process.env.CI

export default defineConfig({
  testDir: './tests/e2e',
  forbidOnly: isCI,
  retries: isCI ? 1 : 0,
  reporter: isCI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: {
    baseURL: 'http://localhost:4791',
    channel: isCI ? undefined : 'chrome',
    viewport: { width: 1100, height: 720 },
    screenshot: 'only-on-failure'
  },
  webServer: {
    command: 'npx vite build ui --mode mock && npx vite preview ui --port 4791 --strictPort',
    url: 'http://localhost:4791',
    // Own port, own server: never test against another app on this PC.
    reuseExistingServer: false,
    timeout: 120_000
  }
})
