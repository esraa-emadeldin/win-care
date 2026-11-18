// `npm run design:compare` — puts each screen next to its panel in the approved
// ChatGPT mockup (design/chatgpt/v1-mockups/images/01.png), so the look can be
// checked, not just the behaviour. Output: design/compare/<screen>.png
import { readFileSync } from 'node:fs'
import { expect, test, type Page } from '@playwright/test'

const BOARD = 'design/chatgpt/v1-mockups/images/01.png'
const OUT = 'design/compare'
/** Scale from mockup-board pixels to our viewport. */
const SCALE = 1.6

interface Screen {
  name: string
  /** Panel position on the 1536x1024 board: x, y, width, height. */
  crop: [number, number, number, number]
  open: (page: Page) => Promise<void>
}

const screens: Screen[] = [
  {
    name: 'home',
    crop: [40, 15, 865, 370],
    open: async (page) => {
      await page.goto('/')
      await page.getByRole('button', { name: 'Scan' }).click()
      await expect(page.getByRole('button', { name: 'Clean' })).toBeVisible()
    }
  },
  {
    name: 'apps',
    crop: [20, 415, 740, 285],
    open: async (page) => {
      await page.goto('/#/apps')
      await expect(page.getByText('Google Chrome')).toBeVisible()
    }
  },
  {
    name: 'performance',
    crop: [780, 415, 740, 285],
    open: async (page) => {
      await page.goto('/#/performance')
      await expect(page.getByText('Spotify').first()).toBeVisible()
    }
  },
  {
    name: 'review',
    crop: [20, 710, 740, 225],
    open: async (page) => {
      await page.goto('/')
      await page.getByRole('button', { name: 'Scan' }).click()
      await page.getByRole('button', { name: 'Clean' }).click()
      await expect(page.getByRole('heading', { name: /Review/ })).toBeVisible()
    }
  },
  {
    name: 'space',
    crop: [780, 710, 740, 225],
    open: async (page) => {
      await page.goto('/#/space')
      await expect(page.getByTitle(/^Windows:/)).toBeVisible()
    }
  },
  {
    name: 'tray',
    crop: [1160, 20, 290, 345],
    open: async (page) => {
      await page.goto('/#/tray')
      await expect(page.getByText('CPU')).toBeVisible()
      await page.waitForTimeout(4500) // a few vitals samples for the sparklines
    }
  }
]

for (const s of screens) {
  test(s.name, async ({ page }) => {
    const [x, y, w, h] = s.crop
    const width = Math.round(w * SCALE)
    const height = Math.round(h * SCALE)
    await page.setViewportSize({ width, height })
    await s.open(page)
    await page.waitForTimeout(300)
    const ours = (await page.screenshot()).toString('base64')

    // Side by side: the mockup panel (cropped from the board) | our screen.
    const board = readFileSync(BOARD).toString('base64')
    await page.setViewportSize({ width: width * 2 + 48, height: height + 72 })
    await page.setContent(`
      <style>
        body { margin: 0; font: 600 18px 'Segoe UI'; background: #222; color: #fff; display: flex; gap: 16px; padding: 16px; }
        figure { margin: 0; } figcaption { margin-bottom: 8px; }
        .mock { width: ${width}px; height: ${height}px;
          background: url(data:image/png;base64,${board}) no-repeat;
          background-size: ${1536 * SCALE}px ${1024 * SCALE}px;
          background-position: -${x * SCALE}px -${y * SCALE}px; }
      </style>
      <figure><figcaption>Mockup</figcaption><div class="mock"></div></figure>
      <figure><figcaption>win-care</figcaption><img src="data:image/png;base64,${ours}" width="${width}" height="${height}"></figure>`)
    await page.screenshot({ path: `${OUT}/${s.name}.png` })
  })
}
