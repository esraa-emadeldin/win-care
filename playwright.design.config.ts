import { defineConfig } from '@playwright/test'
import base from './playwright.config'

// Design comparison only (npm run design:compare); not part of CI.
export default defineConfig({
  ...base,
  testDir: './tests/design',
  retries: 0,
  // A screen that never settles fails in a minute instead of hanging.
  timeout: 60_000,
  reporter: 'list'
})
