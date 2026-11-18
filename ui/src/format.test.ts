import { describe, expect, it } from 'vitest'
import { formatBytes } from './format'

describe('formatBytes', () => {
  it('uses 1024-based units with one decimal', () => {
    expect(formatBytes(1932735283)).toBe('1.8 GB')
    expect(formatBytes(1536)).toBe('1.5 KB')
  })
  it('shows zero as bytes', () => {
    expect(formatBytes(0)).toBe('0 B')
  })
})
