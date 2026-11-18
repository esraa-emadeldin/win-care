// The start-up splash lives in index.html so it shows before any JavaScript.
// These remove it: no minimum display time, so it never slows start-up.

const FADE_MS = 250

function splash(): HTMLElement | null {
  return document.getElementById('splash')
}

/** The tray popup never shows the splash. */
export function removeSplashNow(): void {
  splash()?.remove()
}

/** Fades the splash out once the app is on screen, then removes it. */
export function hideSplash(): void {
  const el = splash()
  if (!el) return
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    el.remove()
    return
  }
  el.classList.add('done')
  el.addEventListener('transitionend', () => el.remove(), { once: true })
  // In case the transition event never fires (e.g. a hidden window).
  window.setTimeout(() => el.remove(), FADE_MS + 100)
}
