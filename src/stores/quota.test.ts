import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { getQuota, getCachedQuota } from '../services/tauri'
import { useQuotaStore } from './quota'
vi.mock('../services/tauri', () => ({ getQuota: vi.fn(), getCachedQuota: vi.fn() }))
beforeEach(() => { setActivePinia(createPinia()); vi.resetAllMocks() })
describe('Quota store', () => {
  it('后台异常保留已成功取得的额度并释放刷新状态', async () => {
    const store = useQuotaStore()
    store.data = { source: 'CLI', stale: false, updatedAt: '2026-10-16T00:00:00Z', fiveHour: { label: '5h', usedPercent: 25 } }
    vi.mocked(getQuota).mockRejectedValue(new Error('IPC disconnected'))
    await store.refresh()
    expect(store.loading).toBe(false)
    expect(store.data.fiveHour?.usedPercent).toBe(25)
    expect(store.data.stale).toBe(true)
    expect(store.data.error?.retryable).toBe(true)
  })
  it('缓存读取失败仍可发起实时刷新', async () => {
    const store = useQuotaStore()
    vi.mocked(getCachedQuota).mockRejectedValue(new Error('cache unavailable'))
    vi.mocked(getQuota).mockResolvedValue({ source: 'CLI', stale: false })
    await store.restore()
    await store.refresh()
    expect(getQuota).toHaveBeenCalledTimes(1)
  })
})
