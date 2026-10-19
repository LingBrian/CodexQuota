import { defineStore } from 'pinia'
import { getQuota, getCachedQuota } from '../services/tauri'
import type { QuotaSnapshot } from '../types'
export const useQuotaStore = defineStore('quota', {
  state: () => ({ data: { source: '尚未查询', stale: false } as QuotaSnapshot, loading: false }),
  actions: {
    async restore() { try { const cached = await getCachedQuota(); if (cached) this.data = cached } catch { /* A failed cache read must not prevent a live refresh. */ } },
    async refresh() {
      if (this.loading) return
      this.loading = true
      try { this.data = await getQuota() }
      catch { this.data = { ...this.data, stale: !!this.data.updatedAt, error: { code: 'internal', message: '无法连接后台查询服务，请重试', retryable: true } } }
      finally { this.loading = false }
    }
  }
})
