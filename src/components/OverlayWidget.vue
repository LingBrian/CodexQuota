<script setup lang="ts">
import { computed } from 'vue'
import { t } from '../i18n'
import type { AppSettings, QuotaSnapshot, WindowQuota } from '../types'
import { remainingPercent } from '../utils/quota'
const props = defineProps<{settings: AppSettings; snapshot: QuotaSnapshot; now: number}>()
const style = computed(() => {
  const overlay = props.settings.overlay
  const color = overlay?.background ?? '#1b1e23'
  const alpha = overlay?.backgroundOpacity ?? 0.94
  return {
    background: `rgba(${parseInt(color.slice(1,3),16)},${parseInt(color.slice(3,5),16)},${parseInt(color.slice(5,7),16)},${alpha})`,
    color: props.settings.overlayTextColor, opacity: props.settings.overlayOpacity,
    fontSize: `${props.settings.overlayFontSize}px`, fontWeight: overlay?.fontWeight ?? 600,
    borderRadius: `${Math.min(overlay?.cornerRadius ?? 10, 24)}px`,
    gap: '2px', flexDirection: (overlay?.reverse ? 'column-reverse' : 'column') as 'column'|'column-reverse',
    '--column-gap': `${Math.min(overlay?.gap ?? 8, 12)}px`,
  }
})
const windows = computed(() => [
  {label:'5h', value:props.snapshot.fiveHour, show:props.settings.overlay?.showFive !== false, color:'#7be2b9'},
  {label:'7d', value:props.snapshot.sevenDay, show:props.settings.overlay?.showSeven !== false, color:'#b0a3f5'},
].filter(window => window.show))
function percentage(quota?: WindowQuota | null) {
  const remaining = remainingPercent(quota ?? undefined)
  return remaining == null ? undefined : props.settings.overlay?.showUsed ? 100-remaining : remaining
}
function countdown(reset?: string | null) {
  if (!reset || !Number.isFinite(Date.parse(reset))) return ''
  const seconds = Math.ceil((Date.parse(reset) - props.now)/1000)
  return seconds <= 0 ? t('等待重新查询') : `${Math.floor(seconds/3600)}h${Math.floor(seconds%3600/60)}m`
}
function resetTime(reset?: string | null) {
  if (!reset || !Number.isFinite(Date.parse(reset))) return '到期时间未知'
  const date = new Date(reset)
  return `${String(date.getMonth()+1).padStart(2,'0')}/${String(date.getDate()).padStart(2,'0')} ${String(date.getHours()).padStart(2,'0')}:${String(date.getMinutes()).padStart(2,'0')}`
}
</script>
<template>
  <div class="overlay-widget" :class="{ 'widget-stale':snapshot.stale, 'widget-transparent':settings.overlay?.backgroundOpacity === 0, [`preset-${settings.overlay?.displayStyle ?? 'bars'}`]:true }" :style="style" :aria-label="t('额度组件')" :title="snapshot.stale ? t('显示缓存数据，可能已过期') : t('额度组件')">
    <div v-for="window in windows" :key="window.label" class="widget-metric" :data-window="window.label" :style="{'--indicator': (remainingPercent(window.value ?? undefined) ?? 100) < 20 ? settings.overlay?.lowColor || '#ff8585' : window.color}">
      <span class="widget-label">{{window.label}}<span v-if="snapshot.stale" class="widget-cache" :aria-label="t('缓存数据')">•</span></span>
      <progress v-if="settings.overlay?.progress && percentage(window.value) != null" :value="percentage(window.value)" max="100" :aria-label="`${window.label} ${t(settings.overlay?.showUsed ? '已使用' : '剩余额度')}`"/>
      <span v-else-if="settings.overlay?.progress" class="widget-unknown-track" :aria-label="`${window.label} ${t('暂无数据')}`"></span>
      <span v-else class="widget-no-track"></span>
      <span class="widget-value">{{percentage(window.value) == null ? '—' : percentage(window.value)?.toFixed(0)+'%'}}</span>
      <span class="widget-countdown" :title="countdown(window.value?.resetAt)">{{resetTime(window.value?.resetAt)}}</span>
    </div>
    <span v-if="!windows.length" class="widget-empty">{{t('未选择显示指标')}}</span>
  </div>
</template>
