import { expect, it } from 'vitest'
import { resolvedTheme } from './preferences'
it('系统主题变化时自动跟随，显式偏好优先', () => {
  expect(resolvedTheme('system', true)).toBe('dark')
  expect(resolvedTheme('system', false)).toBe('light')
  expect(resolvedTheme('light', true)).toBe('light')
  expect(resolvedTheme('dark', false)).toBe('dark')
})
