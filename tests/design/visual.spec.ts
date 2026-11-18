// `npm run design:check` — the approved look, locked. Fails when a screen
// drifts from its baseline in tests/design/visual.spec.ts-snapshots/.
// After an intended change: `npm run design:check -- --update-snapshots`.
// Local only (not in CI): font rendering differs between machines.
import { expect, test, type Page } from '@playwright/test'

const screens: { name: string; open: (page: Page) => Promise<void> }[] = [
  {
    name: 'home',
    open: async (page) => {
      await page.goto('/')
      await page.getByRole('button', { name: 'Scan' }).click()
      await expect(page.getByRole('button', { name: 'Clean' })).toBeVisible()
    }
  },
  {
    name: 'review',
    open: async (page) => {
      await page.goto('/')
      await page.getByRole('button', { name: 'Scan' }).click()
      await page.getByRole('button', { name: 'Clean' }).click()
      await expect(page.getByRole('heading', { name: /Review/ })).toBeVisible()
    }
  },
  {
    name: 'apps',
    open: async (page) => {
      await page.goto('/#/apps')
      await expect(page.getByText('Google Chrome')).toBeVisible()
    }
  },
  {
    name: 'performance',
    open: async (page) => {
      await page.goto('/#/performance')
      await expect(page.getByText('Spotify').first()).toBeVisible()
    }
  },
  {
    name: 'space',
    open: async (page) => {
      await page.goto('/#/space')
      await expect(page.getByTitle(/^Windows:/)).toBeVisible()
    }
  },
  {
    name: 'settings',
    open: async (page) => {
      await page.goto('/#/settings')
      await expect(page.getByText('Start with Windows')).toBeVisible()
    }
  }
]

for (const s of screens) {
  test(s.name, async ({ page }) => {
    await page.setViewportSize({ width: 1100, height: 720 })
    await s.open(page)
    // Wait for the start-up splash to fade.
    await expect(page.locator('#splash')).toHaveCount(0)
    await expect(page).toHaveScreenshot(`${s.name}.png`, { maxDiffPixelRatio: 0.01 })
  })
}
