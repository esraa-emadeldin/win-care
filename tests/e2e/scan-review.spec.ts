// The core flow against the mocked backend (ui/src/mock.ts): scan, review,
// move to the Recycle Bin, see what was skipped and why.
import { expect, test } from '@playwright/test'

async function scanAndClean(page: import('@playwright/test').Page) {
  await page.goto('/')
  await page.getByRole('button', { name: 'Scan' }).click()
  // The card shows all junk found; Review counts only what the bin can take.
  await expect(page.getByTestId('card-cleanup')).toContainText('3.5 GB')
  await page.getByRole('button', { name: 'Clean' }).click()
  await expect(page.getByRole('heading', { name: 'Review before deleting' })).toBeVisible()
}

test('scan then review shows preselected total', async ({ page }) => {
  await scanAndClean(page)
  await expect(page.getByRole('button', { name: /Move to Recycle Bin/ })).toHaveText(
    'Move to Recycle Bin (510.0 MB)'
  )
  await expect(page.getByText('Nothing will be deleted until you confirm.')).toBeVisible()
})

test('old installers start unchecked', async ({ page }) => {
  await scanAndClean(page)
  // Groups start collapsed, as in the mockup.
  await page.getByRole('button', { name: 'Show Old installers in Downloads' }).click()
  await page.getByRole('button', { name: 'Show Browser caches' }).click()
  await expect(page.getByRole('checkbox', { name: 'setup-old.exe' })).not.toBeChecked()
  await expect(page.getByRole('checkbox', { name: 'session-cache' })).toBeChecked()
})

test('apply shows skipped reasons', async ({ page }) => {
  await scanAndClean(page)
  await page.getByRole('button', { name: /Move to Recycle Bin/ }).click()
  await expect(page.getByText('Moved 2 items, freed 500.0 MB')).toBeVisible()
  await expect(page.getByText('in use')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Open Recycle Bin' })).toBeVisible()
})

test('unrecyclable item shows badge and is not counted', async ({ page }) => {
  await scanAndClean(page)
  await expect(page.getByText('too large for the Recycle Bin')).toBeVisible()
  await expect(page.getByRole('checkbox', { name: 'huge.dmp' })).toBeDisabled()
  await expect(page.getByRole('button', { name: /Move to Recycle Bin/ })).toHaveText(
    'Move to Recycle Bin (510.0 MB)'
  )
})
