import { describe, expect, it } from 'vitest'
import type { Finding } from '../types'
import { reviewItems, selectedTotal } from './scan'

function finding(path: string, bytes: number, extra: Partial<Finding> = {}): Finding {
  return {
    id: path,
    category: 'userTemp',
    path,
    bytes,
    isDir: false,
    modifiedMs: 0,
    preselected: true,
    selectable: true,
    note: null,
    ...extra
  }
}

describe('selectedTotal', () => {
  it('counts only selected, selectable, recyclable items', () => {
    const items = reviewItems(
      [
        finding('a', 100),
        finding('b', 50, { preselected: false }),
        finding('c', 30, { selectable: false }),
        finding('d', 1000)
      ],
      new Set(['d'])
    )
    expect(items.map((i) => i.selected)).toEqual([true, false, false, false])
    expect(selectedTotal(items)).toBe(100)

    items[1]!.selected = true
    expect(selectedTotal(items)).toBe(150)
  })
})
