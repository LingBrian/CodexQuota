<script setup lang="ts">
import { t } from '../i18n'
import type { WindowQuota } from '../types'
import { quotaLabel, remainingPercent } from '../utils/quota'
import UiIcon from './UiIcon.vue'
defineProps<{ quota?: WindowQuota | null; label: string; stale?: boolean; now: number }>()
function resetText(text?: string | null) { return text && Number.isFinite(Date.parse(text)) ? new Date(text).toLocaleString() : t('暂无数据') }
function countdown(text: string | undefined | null, now: number) {
  if (!text || !Number.isFinite(Date.parse(text))) return t('暂无数据')
  const seconds = Math.ceil((Date.parse(text) - now) / 1000)
  if (seconds <= 0) return t('等待重新查询')
  return `${Math.floor(seconds / 3600)}h ${Math.floor(seconds % 3600 / 60)}m`
}
</script>
<template>
  <article class="card quota-card" :class="{weekly:label === '7d', low:(remainingPercent(quota ?? undefined) ?? 100) < 20}" :aria-label="`${label} ${t('额度')}`">
    <div class="card-heading"><div class="quota-kind"><span><UiIcon :name="label === '5h' ? 'clock' : 'pulse'"/></span>{{ t(label === '5h' ? '5 小时窗口' : '7 天窗口') }}</div><span v-if="stale" class="badge">{{ t('缓存数据') }}</span><span v-else class="subtle-label">{{label.toUpperCase()}} WINDOW</span></div>
    <div class="quota-reading"><strong>{{ quotaLabel(quota ?? undefined) }}</strong><span class="muted">{{ t('剩余额度') }}</span></div>
    <div v-if="remainingPercent(quota ?? undefined) != null" class="bar" role="progressbar" :aria-label="label" :aria-valuenow="remainingPercent(quota ?? undefined)" aria-valuemin="0" aria-valuemax="100"><i :style="{ width: remainingPercent(quota ?? undefined) + '%' }" /></div>
    <div v-else class="bar unknown" :aria-label='t("暂无额度数据")' />
    <div class="quota-meta"><span>{{ t('已使用：') }}{{ remainingPercent(quota ?? undefined) == null ? t('暂无数据') : `${quota?.usedPercent}%` }}</span><span>{{ t('距重置：') }}{{ countdown(quota?.resetAt, now) }}</span></div>
    <div class="quota-reset"><span>{{t('重置时间')}}</span><time :datetime="quota?.resetAt || undefined">{{resetText(quota?.resetAt)}}</time></div>
  </article>
</template>
