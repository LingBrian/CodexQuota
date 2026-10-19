<script setup lang="ts">
import { t } from '../i18n'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
const path = ref('')
const result = ref('')
async function exportReport() {
  try { result.value = await invoke('export_diagnostics', { path: path.value }) }
  catch (reason) { result.value = String(reason) }
}
</script>
<template><section><h3>{{ t("脱敏诊断") }}</h3><p>{{ t("仅导出版本、系统、账号数量、缓存状态与固定错误码，不含账号名称、路径或凭据。") }}</p><label>{{ t("完整导出路径") }}<input class="path-input" v-model="path" placeholder="D:\Backup\report.codexquota-diagnostics.json"></label><button @click="exportReport">{{ t("导出诊断报告") }}</button><p role="status">{{ result }}</p></section></template>
