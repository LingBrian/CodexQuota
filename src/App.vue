<script setup lang="ts">
import { t } from './i18n'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { applyAppearance } from './preferences'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getVersion } from '@tauri-apps/api/app'
import { useQuotaStore } from './stores/quota'
import { getSettings, saveSettings, setOverlay, cancelRefresh } from './services/tauri'
import type { AppSettings, QuotaSnapshot } from './types'
import UiIcon from './components/UiIcon.vue'
import QuotaCard from './components/QuotaCard.vue'
import AccountsView from './components/AccountsView.vue'
import ReminderEditor from './components/ReminderEditor.vue'
import OverlayWidget from './components/OverlayWidget.vue'
import OverlayEditor from './components/OverlayEditor.vue'
import DiagnosticsPanel from './components/DiagnosticsPanel.vue'
import MaintenancePanel from './components/MaintenancePanel.vue'
import StartupToggle from './components/StartupToggle.vue'

const overlay = location.hash === '#overlay'
document.documentElement.classList.toggle('overlay-document', overlay)
const quota = useQuotaStore()
const page = ref('仪表盘')
const error = ref('')
const version = ref('')
const now = ref(Date.now())
const settings = ref<AppSettings>({ settingsVersion: 2, refreshSeconds: 60, quotaCommand: 'codex-switch', overlayVisible: false, overlayTextColor: '#ffffff', overlayFontSize: 14, overlayOpacity: 0.92 })
const nav = ['仪表盘', '账号管理', '任务栏显示', '额度提醒', '设置', '关于']
const status = computed(() => quota.loading ? '正在同步…' : quota.data.error?.message || (quota.data.stale ? '显示缓存数据，可能已过期' : quota.data.updatedAt ? '已同步' : '尚未查询'))
const navIcons: Record<string, string> = { '仪表盘':'dashboard', '账号管理':'accounts', '任务栏显示':'display', '额度提醒':'bell', '设置':'settings', '关于':'info' }
const descriptions: Record<string, string> = {
  '仪表盘':'掌握每一份额度，专注下一次创造。', '账号管理':'在本地管理账号，安全切换工作状态。',
  '任务栏显示':'让额度常驻视线，不打断你的工作。', '额度提醒':'只在需要的时候，给你恰好的提醒。',
  '设置':'按你的习惯，调整每一处细节。', '关于':'轻量、本地优先，始终透明。',
}
const healthy = computed(() => !quota.data.error && !quota.data.stale && !!quota.data.updatedAt)
const syncTime = computed(() => quota.data.updatedAt ? new Date(quota.data.updatedAt).toLocaleTimeString([], { hour:'2-digit', minute:'2-digit', second:'2-digit' }) : '—')
const saved = ref(false)
let savedTimer: ReturnType<typeof setTimeout> | undefined
const cleanup: UnlistenFn[] = []
const systemTheme = matchMedia('(prefers-color-scheme: dark)')
const updateAppearance = () => applyAppearance(settings.value, systemTheme.matches)
watch(() => [settings.value.language, settings.value.theme, settings.value.accentColor], updateAppearance, { immediate: true })
let clock: ReturnType<typeof setInterval> | undefined
let saveQueue: Promise<void> = Promise.resolve()
let saveSequence = 0
let savingCount = 0
async function apply() {
  const sequence = ++saveSequence
  const snapshot = JSON.parse(JSON.stringify(settings.value)) as AppSettings
  savingCount++
  error.value = ''
  saveQueue = saveQueue.then(async () => {
    try { const result = await saveSettings(snapshot); if(sequence===saveSequence){settings.value=result; saved.value = true; if(savedTimer) clearTimeout(savedTimer); savedTimer = setTimeout(() => { saved.value = false }, 2500)} }
    catch (reason) { error.value = typeof reason === 'string' ? reason : '设置保存失败，请重试' }
    finally { savingCount-- }
  })
  await saveQueue
}
onMounted(async () => {
  systemTheme.addEventListener('change', updateAppearance)
  try {
    cleanup.push(await listen<QuotaSnapshot>('quota-updated', event => { quota.data = event.payload }))
    cleanup.push(await listen<AppSettings>('settings-updated', event => { if(!savingCount) settings.value = event.payload }))
    cleanup.push(await listen<string>('application-error', event => { error.value = event.payload }))
    cleanup.push(await listen<string>('navigate', event => { if (nav.includes(event.payload)) page.value = event.payload }))
    settings.value = await getSettings()
    error.value = await invoke<string | null>('settings_warning') ?? ''
    version.value = await getVersion()
    await quota.restore()
    if (overlay) {
      await nextTick()
      const widget=document.querySelector<HTMLElement>('.overlay-widget')
      const metrics = [...document.querySelectorAll<HTMLElement>('.widget-metric')]
      const layoutValid = !!widget && document.documentElement.scrollWidth <= innerWidth + 1 && getComputedStyle(widget).paddingLeft==='0px' && metrics.length===2 && metrics.every(metric => metric.scrollWidth <= metric.clientWidth + 1 && metric.scrollHeight <= metric.clientHeight + 1 && getComputedStyle(metric).flexDirection==='row' && getComputedStyle(metric).alignItems==='center')
      const backgroundMatches=!!widget && getComputedStyle(widget).backgroundColor===`rgba(${parseInt(settings.value.overlay?.background.slice(1,3)||'1b',16)}, ${parseInt(settings.value.overlay?.background.slice(3,5)||'1e',16)}, ${parseInt(settings.value.overlay?.background.slice(5,7)||'23',16)}, ${settings.value.overlay?.backgroundOpacity ?? 0.9})`
      const resetTimesVisible=metrics.length===2 && [...document.querySelectorAll<HTMLElement>('.widget-countdown')].length===2 && ![...document.querySelectorAll<HTMLElement>('.widget-countdown')].some(item=>item.textContent?.includes('未知'))
      await invoke('overlay_loaded', { layoutValid, backgroundMatches, resetTimesVisible })
    }
    if (!overlay) await quota.refresh()
    if (!overlay && await invoke<boolean>('is_smoke_test')) {
      // Exercise the real IPC command path that previously ran synchronously.
      await setOverlay(true)
      await setOverlay(false)
      await setOverlay(true)
      await nextTick()
      const progressBars=document.querySelectorAll('.cards [role="progressbar"]').length
      const dashboardFits=document.documentElement.scrollWidth <= innerWidth + 1
      page.value='账号管理'; await nextTick()
      const accountRows=document.querySelectorAll('[data-account]').length
      const choose=document.querySelector('article button') as HTMLButtonElement | null
      choose?.click(); await nextTick(); await nextTick()
      const dialog=document.querySelector('dialog') as HTMLDialogElement | null
      const modalVerified=dialog?.open === true && dialog.matches(':modal')
      dialog?.dispatchEvent(new Event('cancel', {cancelable:true})); await nextTick(); await nextTick()
      const originalLanguage=settings.value.language, originalTheme=settings.value.theme
      settings.value.language='en'; settings.value.theme='light'; await nextTick()
      let presentationVerified=dashboardFits && modalVerified && document.querySelector('nav')?.textContent?.includes('Dashboard') === true && document.documentElement.dataset.theme === 'light'
      for (const target of nav) {
        page.value=target; await nextTick(); await nextTick()
        presentationVerified = presentationVerified && document.documentElement.scrollWidth <= innerWidth + 1
      }
      settings.value.language=originalLanguage; settings.value.theme=originalTheme
      page.value='仪表盘'; await nextTick()
      await invoke('finish_smoke_test', { progressBars, accountRows, presentationVerified })
    }
    clock = setInterval(() => { now.value = Date.now() }, 1000)
  } catch { error.value = '无法连接桌面后台，请关闭应用后重新打开' }
})
onUnmounted(() => { systemTheme.removeEventListener('change', updateAppearance); cleanup.forEach(stop => stop()); if (clock) clearInterval(clock); if(savedTimer) clearTimeout(savedTimer) })
</script>
<template>
  <OverlayWidget v-if="overlay" :settings="settings" :snapshot="quota.data" :now="now" @contextmenu.prevent="invoke('overlay_menu')"/>
  <main v-else class="app-shell">
    <aside class="sidebar">
      <div class="brand"><span class="brand-mark">C<span></span></span><div>CodexQuota<small>YOUR QUOTA, AT A GLANCE</small></div></div>
      <div class="nav-caption">{{ t('工作空间') }}</div>
      <nav :aria-label="t('主导航')"><button v-for="item in nav" :key="item" :class="{ active: page === item }" :aria-current="page === item ? 'page' : undefined" @click="page = item"><UiIcon :name="navIcons[item]"/><span>{{ t(item) }}</span><span v-if="page === item" class="nav-indicator"></span></button></nav>
      <div class="sidebar-bottom"><div class="local-note"><UiIcon name="shield"/><div>{{ t('本地优先') }}<small>{{ t('凭据始终留在你的设备上') }}</small></div></div><button class="sidebar-account" @click="page = '账号管理'"><span class="avatar">{{ quota.data.account?.slice(0,1).toUpperCase() || 'C' }}</span><span>{{ quota.data.account || t('尚未选择账号') }}<small>v{{ version }}</small></span><UiIcon name="chevron"/></button></div>
    </aside>
    <div class="workspace">
      <div class="topbar"><span>CodexQuota <span class="breadcrumb-divider">/</span> {{ t(page) }}</span><span class="connection-pill" :class="{ connected: healthy, warning: quota.data.stale || quota.data.error }"><i></i>{{ t(status) }}</span></div>
      <section class="content">
        <header class="page-header"><div><p class="eyebrow">{{ page === '仪表盘' ? 'OVERVIEW' : 'PREFERENCES & WORKSPACE' }}</p><h1>{{ t(page === '仪表盘' ? '额度总览' : page) }}</h1><p class="page-description">{{ t(descriptions[page]) }}</p></div><div class="header-actions"><button v-if="quota.loading" @click="cancelRefresh">{{ t('取消查询') }}</button><button class="primary" :disabled="quota.loading" @click="quota.refresh()"><UiIcon name="refresh" :class="{ spinning: quota.loading }"/>{{ t(quota.loading ? '刷新中…' : '立即刷新') }}</button></div></header>
        <p v-if="error" class="error-banner" role="alert">{{ error }}</p><p v-if="saved" class="save-toast" role="status">{{t('设置已保存')}}</p>
        <template v-if="page === '仪表盘'">
          <div class="account-strip"><span class="avatar">{{ quota.data.account?.slice(0,1).toUpperCase() || 'C' }}</span><div><small>{{ t('当前账号') }}</small><strong>{{ quota.data.account || t('尚未选择账号') }}</strong></div><button @click="page = '账号管理'">{{ t('管理账号') }}<UiIcon name="arrow"/></button></div>
          <div class="cards"><QuotaCard :quota="quota.data.fiveHour" label="5h" :stale="quota.data.stale" :now="now"/><QuotaCard :quota="quota.data.sevenDay" label="7d" :stale="quota.data.stale" :now="now"/></div>
          <article class="card sync-card"><div class="section-heading"><h2>{{ t('同步信息') }}</h2><span class="subtle-label">LIVE CONNECTION</span></div><div class="sync-grid"><div><UiIcon name="clock"/><span>{{t('最后成功更新')}}<strong>{{ syncTime }}</strong></span></div><div><UiIcon name="refresh"/><span>{{t('刷新间隔')}}<strong>{{settings.refreshSeconds}} {{t('秒')}}</strong></span></div><div><UiIcon name="shield"/><span>{{t('数据来源')}}<strong>{{quota.data.updatedAt ? quota.data.source : t('等待同步')}}</strong></span></div></div><p v-if="quota.data.error" class="inline-error">{{quota.data.error.message}}</p></article>
          <button class="companion-link" @click="page = '任务栏显示'"><span class="feature-icon"><UiIcon name="display"/></span><span><strong>{{ t('不打开窗口，也能掌握额度') }}</strong><small>{{ t('配置你的任务栏额度组件，让信息触手可及。') }}</small></span><UiIcon name="arrow"/></button>
        </template>
        <template v-else-if="page === '任务栏显示'">
          <article class="card companion-preview"><div class="section-heading"><div><h2>{{ t('实时预览') }}</h2><p>{{ t('使用当前真实额度，样式修改后即时应用。') }}</p></div><label class="switch-label"><input type="checkbox" v-model="settings.overlayVisible" @change="apply">{{t('显示任务栏组件')}}</label></div><div class="preview-stage"><div class="preview-desktop-label">{{t('你的工作空间')}}</div><div class="preview-taskbar"><span class="taskbar-apps"><i></i><i></i><i></i></span><div v-if="settings.overlayVisible" class="widget-preview" :style="{width: (settings.overlay?.width || 300) + 'px', height: (settings.overlay?.height || 46) + 'px'}"><OverlayWidget :settings="settings" :snapshot="quota.data" :now="now"/></div><span v-else>组件已隐藏</span><span class="preview-clock">{{ new Date(now).toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'}) }}</span></div></div><p class="hint">任务栏内模式将组件放在任务栏矩形范围内。可在下方调整沿栏位置，或切换桌面浮窗模式。</p></article>
          <article class="card"><div class="section-heading"><div><h2>{{t('文字与透明度')}}</h2><p>{{t('为你的桌面搭配合适的阅读体验。')}}</p></div></div><div class="form-grid"><label>{{t('字体大小')}}<input type="number" v-model.number="settings.overlayFontSize" min="10" max="36" @change="apply"></label><label>{{t('字体颜色')}}<input type="color" v-model="settings.overlayTextColor" @change="apply"></label><label>{{t('整体透明度')}}<input type="range" v-model.number="settings.overlayOpacity" min="0.2" max="1" step="0.05" @change="apply"></label></div></article>
          <OverlayEditor :value="settings" @save="apply"/>
        </template>
        <template v-else-if="page === '设置'">
          <article class="card"><div class="section-heading"><div><h2>{{t('外观与启动')}}</h2><p>{{t('按你的习惯，调整每一处细节。')}}</p></div></div><div class="form-grid"><label>{{t('语言')}}<select v-model="settings.language" @change="apply"><option value="zh-CN">{{t('简体中文')}}</option><option value="en">English</option></select></label><label>{{t('主题')}}<select v-model="settings.theme" @change="apply"><option value="system">{{t('跟随系统')}}</option><option value="dark">{{t('深色')}}</option><option value="light">{{t('浅色')}}</option></select></label><label>{{t('强调色')}}<input type="color" v-model="settings.accentColor" @change="apply"></label></div><StartupToggle/></article>
          <article class="card"><div class="section-heading"><div><h2>{{t('查询设置')}}</h2><p>{{t('使用随包 CLI 查询，保留上游缓存以减少重复请求。')}}</p></div></div><div class="form-grid"><label>{{t('刷新间隔（秒）')}}<input type="number" v-model.number="settings.refreshSeconds" min="60" max="3600"></label><label class="wide-field">{{t('CLI 路径')}}<input v-model="settings.quotaCommand" :placeholder="t('codex-switch 或 codex-switch.exe 的完整路径')"></label></div><div class="form-footer"><button class="primary" @click="apply">{{t('保存设置')}}</button></div></article>
          <article class="card"><MaintenancePanel @imported="settings = $event"/></article>
        </template>
        <AccountsView v-else-if="page === '账号管理'"/>
        <ReminderEditor v-else-if="page === '额度提醒' && settings.reminders" :value="settings.reminders" @save="apply"/>
        <template v-else-if="page === '关于'"><article class="card about-card"><span class="brand-mark large">C<span></span></span><h2>CodexQuota</h2><p>{{t('轻量、本地优先，始终透明。')}}</p><span class="version-tag">v{{version}} · Windows x64</span><div class="about-facts"><span>MIT License</span><span>Tauri 2 + Vue 3</span><span>{{t('不收集遥测')}}</span></div></article><article class="card"><DiagnosticsPanel/></article></template>
        <footer class="workspace-footer"><span>CodexQuota <span>·</span> {{t('为专注而设计')}}</span><span>{{t('本地优先')}} <i class="footer-dot"></i></span></footer>
      </section>
    </div>
  </main>
</template>
