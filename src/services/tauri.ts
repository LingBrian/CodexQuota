import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, QuotaSnapshot } from '../types'
export const getQuota = () => invoke<QuotaSnapshot>('refresh_quota')
export const getCachedQuota = () => invoke<QuotaSnapshot | null>('cached_quota')
export const cancelRefresh = () => invoke('cancel_refresh')
export const getSettings = () => invoke<AppSettings>('get_settings')
export const saveSettings = (settings: AppSettings) => invoke<AppSettings>('save_settings', { settings })
export const setOverlay = (visible: boolean) => invoke('set_overlay_visible', { visible })
