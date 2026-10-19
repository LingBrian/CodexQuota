import { afterEach, describe, expect, it } from 'vitest'
import { language, t } from './i18n'
afterEach(() => { language.value = 'zh-CN' })
describe('本地语言切换', () => {
  it('中文默认保留，英语即时切换，不改变内部导航标识', () => {
    expect(t('账号管理')).toBe('账号管理')
    language.value = 'en'
    expect(t('账号管理')).toBe('Accounts')
    expect(t('确认账号操作')).toBe('Confirm account operation')
    expect(t('CodexQuota')).toBe('CodexQuota')
  })
})
