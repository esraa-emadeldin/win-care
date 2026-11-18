import { describe, expect, it } from 'vitest'
import { layout } from './treemap'
import type { SpaceNode } from './types'

const node = (name: string, bytes: number): SpaceNode => ({
  name,
  path: `C:\\${name}`,
  bytes,
  children: []
})

const root: SpaceNode = {
  name: 'C:\\',
  path: 'C:\\',
  bytes: 1000,
  children: [node('a', 500), node('b', 300), node('c', 150), node('d', 50)]
}

describe('treemap layout', () => {
  it('rects fill the area', () => {
    const rects = layout(root, 800, 400)
    const area = rects.reduce((s, r) => s + r.w * r.h, 0)
    expect(Math.abs(area - 800 * 400) / (800 * 400)).toBeLessThan(0.005)
  })

  it('largest first, area proportional to size', () => {
    const rects = layout(root, 800, 400)
    expect(rects.map((r) => r.name)).toEqual(['a', 'b', 'c', 'd'])
    const a = rects[0]!
    expect((a.w * a.h) / (800 * 400)).toBeCloseTo(0.5, 2)
  })

  it('an empty folder gives no rects', () => {
    expect(layout({ ...root, bytes: 0, children: [] }, 800, 400)).toEqual([])
  })
})
