import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import QuotaCard from './QuotaCard.vue'
describe('QuotaCard', () => {
  it('缺失数据没有伪造的 100% 进度条', () => {
    const view = mount(QuotaCard, { props: { label: '5h', now: Date.now() } })
    expect(view.text()).toContain('暂无数据')
    expect(view.find('[role="progressbar"]').exists()).toBe(false)
    expect(view.text()).not.toContain('100%')
  })
  it('显示真实剩余额度、缓存状态与重置倒计时', () => {
    const view = mount(QuotaCard, { props: { label: '5h', quota: { label: '5h', usedPercent: 25, resetAt: '2026-10-16T01:00:00Z' }, stale: true, now: Date.parse('2026-10-16T00:00:00Z') } })
    expect(view.text()).toContain('75%')
    expect(view.text()).toContain('缓存数据')
    expect(view.text()).toContain('1h 0m')
    expect(view.find('[role="progressbar"]').attributes('aria-valuenow')).toBe('75')
  })
})
