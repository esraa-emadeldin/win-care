// Fixes from the final review: refused items are shown as such (I1), and
// unreadable folders are reported (I5).
import { expect, test } from '@playwright/test'

test('a protected pick from the space map is shown as protected, not selected', async ({
  page
}) => {
  await page.goto('/#/space')
  await page.getByTitle(/^Windows:/).click()
  await page.getByRole('button', { name: 'Review for deletion' }).click()

  await expect(page.getByText('protected folder')).toBeVisible()
  await expect(page.getByRole('checkbox', { name: 'Windows' }).last()).not.toBeChecked()
  await expect(page.getByRole('checkbox', { name: 'Windows' }).last()).toBeDisabled()
  await expect(page.getByRole('button', { name: /Move to Recycle Bin/ })).toBeDisabled()
})

test('home says how many folders could not be read', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('button', { name: 'Scan' }).click()
  await expect(page.getByText('2 folders could not be read and were skipped')).toBeVisible()
})
