// Squarified treemap layout for the space map (d3-hierarchy does the maths).
import { hierarchy, treemap, treemapSquarify } from 'd3-hierarchy'
import type { SpaceNode } from './types'

export interface Rect {
  path: string
  name: string
  bytes: number
  x: number
  y: number
  w: number
  h: number
}

/** One rectangle per direct child of `node`, largest first, filling w x h. */
export function layout(node: SpaceNode, w: number, h: number): Rect[] {
  if (node.bytes === 0 || node.children.length === 0) return []
  // Only one level is drawn at a time; drilling in shows the next one.
  const flat: SpaceNode = { ...node, children: node.children.map((c) => ({ ...c, children: [] })) }
  const root = hierarchy(flat)
    .sum((d) => (d.children.length === 0 ? d.bytes : 0))
    .sort((a, b) => (b.value ?? 0) - (a.value ?? 0))
  treemap<SpaceNode>().tile(treemapSquarify).size([w, h]).padding(0)(root)
  return (root.children ?? []).map((c) => {
    const r = c as typeof c & { x0: number; y0: number; x1: number; y1: number }
    return {
      path: c.data.path,
      name: c.data.name,
      bytes: c.data.bytes,
      x: r.x0,
      y: r.y0,
      w: r.x1 - r.x0,
      h: r.y1 - r.y0
    }
  })
}
