// Applications, Performance and Settings against the mocked backend.
import { expect, test } from '@playwright/test'

test('startup toggle only for per-user items', async ({ page }) => {
  await page.goto('/#/performance')
  await expect(page.getByRole('switch', { name: 'Spotify' })).toBeVisible()
  await expect(page.getByRole('switch', { name: 'SecurityHealth' })).toHaveCount(0)
  await expect(page.getByText('Needs admin — change it in Task Manager')).toBeVisible()
  await expect(page.getByText('Scheduled task — open Task Scheduler')).toBeVisible()
})

test('empty recycle bin asks first', async ({ page }) => {
  await page.goto('/#/performance')
  await page.getByRole('checkbox', { name: /Empty Recycle Bin/ }).check()
  await page.getByRole('button', { name: 'Run Maintenance' }).click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toContainText(
    'Permanently delete 1.2 GB in the Recycle Bin? This cannot be undone.'
  )
  await dialog.getByRole('button', { name: 'Cancel' }).click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(page.getByText('Recycle Bin emptied')).toHaveCount(0)
})

test('uninstall leftovers open review', async ({ page }) => {
  await page.goto('/#/apps')
  await page
    .getByRole('row', { name: /Contoso Notes/ })
    .getByRole('button', { name: 'Uninstall' })
    .click()
  await expect(page.getByRole('dialog')).toContainText(
    'Uninstall Contoso Notes? Its own uninstaller will open.'
  )
  await page.getByRole('dialog').getByRole('button', { name: 'Uninstall' }).click()
  await expect(page.getByRole('heading', { name: 'Review before deleting' })).toBeVisible()
  await expect(page.getByRole('checkbox', { name: 'Contoso Notes' }).first()).toBeChecked()
  await expect(page.getByText('Needs admin; not removed in v1')).toBeVisible()
})

test('settings defaults', async ({ page }) => {
  await page.goto('/#/settings')
  await expect(page.getByRole('switch', { name: 'Start with Windows' })).not.toBeChecked()
  // v1 has no permanent-delete switch: everything goes to the Recycle Bin.
  await expect(page.getByRole('switch', { name: /permanently/i })).toHaveCount(0)
  await expect(page.getByRole('switch', { name: 'Check for updates once a day' })).not.toBeChecked()
  await expect(page.getByLabel('Warn when free space is below')).toHaveValue('10')
  await expect(page.getByLabel('Theme')).toHaveValue('system')
})
