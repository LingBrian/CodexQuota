import type { WindowQuota } from '../types'
import { t } from '../i18n'

/** Returns remaining quota only when the upstream supplied a valid used percentage. */
export function remainingPercent(quota?: WindowQuota): number | undefined {
  if (quota?.usedPercent == null || !Number.isFinite(quota.usedPercent) || quota.usedPercent < 0 || quota.usedPercent > 100) return undefined
  return 100 - quota.usedPercent
}

export function quotaLabel(quota?: WindowQuota): string {
  const remaining = remainingPercent(quota)
  return remaining == null ? t('暂无数据') : `${remaining}%`
}
