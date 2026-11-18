// The start-up splash: shown at once, gone as soon as the app is ready,
// never in the tray popup.
import { expect, test } from '@playwright/test'

test('splash shows while loading, then gives way to the app', async ({ page }) => {
  // Hold the app's JavaScript back for a moment, as a slow disk would.
  await page.route('**/assets/*.js', async (route) => {
    await new Promise((r) => setTimeout(r, 800))
    await route.continue()
  })
  await page.goto('/')
  await expect(page.getByRole('status', { name: 'Loading win-care' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Scan' })).toBeVisible()
  await expect(page.getByRole('status', { name: 'Loading win-care' })).toHaveCount(0)
})

test('tray popup skips the splash', async ({ page }) => {
  await page.goto('/#/tray')
  await expect(page.getByRole('button', { name: 'Open app' })).toBeVisible()
  await expect(page.locator('#splash')).toHaveCount(0)
})
