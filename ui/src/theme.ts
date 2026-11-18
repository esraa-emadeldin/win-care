/** Follows the saved theme; "system" leaves it to Windows' light/dark setting. */
export function applyTheme(theme: string): void {
  if (theme === 'light' || theme === 'dark') document.documentElement.dataset.theme = theme
  else delete document.documentElement.dataset.theme
}
