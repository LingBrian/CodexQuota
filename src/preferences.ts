import { language } from './i18n'
import type { AppSettings } from './types'
export function resolvedTheme(theme: string | undefined, systemDark: boolean): 'light' | 'dark' {
  return theme === 'light' || theme === 'dark' ? theme : systemDark ? 'dark' : 'light'
}
export function applyAppearance(settings: AppSettings, systemDark: boolean) {
  language.value = settings.language ?? 'zh-CN'
  document.documentElement.lang = language.value
  document.documentElement.dataset.theme = resolvedTheme(settings.theme, systemDark)
  document.documentElement.style.setProperty('--accent', settings.accentColor ?? '#75dfbb')
}
