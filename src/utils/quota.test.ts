import { describe, expect, it } from 'vitest'
import { quotaLabel, remainingPercent } from './quota'

describe('额度展示', () => {
  it('从已用比例计算剩余比例', () => {
    expect(remainingPercent({ label: '5h', usedPercent: 15 })).toBe(85)
  })

  it('越界输入显示未知，不伪装成满额度或耗尽', () => {
    expect(remainingPercent({ label: '5h', usedPercent: -10 })).toBeUndefined()
    expect(remainingPercent({ label: '7d', usedPercent: 130 })).toBeUndefined()
  })

  it('缺失或无效数据绝不伪装为满额度', () => {
    expect(remainingPercent()).toBeUndefined()
    expect(quotaLabel()).toBe('暂无数据')
    expect(quotaLabel({ label: '5h', usedPercent: Number.NaN })).toBe('暂无数据')
  })
})
