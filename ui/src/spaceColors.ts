// The space map's colours, from the mockup: blue, green, purple, orange for the
// four largest blocks, then quiet tints so small items don't compete.
const MAIN = ['#3b82f6', '#22c55e', '#a855f7', '#f59e0b']
const QUIET = ['#a5b4fc', '#94a3b8', '#c4b5fd', '#93c5fd']
const OTHER = '#9ca3af'

export function spaceColor(index: number, isOther = false): string {
  if (isOther) return OTHER
  return index < MAIN.length ? MAIN[index]! : QUIET[(index - MAIN.length) % QUIET.length]!
}
