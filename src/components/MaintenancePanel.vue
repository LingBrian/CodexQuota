<script setup lang="ts">
import { t } from '../i18n'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppSettings } from '../types'
const emit = defineEmits<{ imported: [AppSettings] }>()
const path = ref('')
const result = ref('')
const busy = ref(false)
async function run(command: string) {
  result.value = ''; busy.value = true
  try {
    const value = await invoke<AppSettings | string>(command, { path: path.value })
    if (command === 'import_settings' && typeof value !== 'string') emit('imported', value)
    result.value = typeof value === 'string' ? value : '设置已导入并应用'
  } catch (reason) { result.value = String(reason) }
  finally { busy.value = false }
}
</script>
<template>
  <section :aria-label='t("本地维护")'><h3>{{ t("本地维护") }}</h3>
    <p>{{ t("设置文件不含认证凭据。导出不会覆盖现有文件；导入会替换当前应用设置。") }}</p>
    <label>{{ t("完整文件路径") }}<input class="path-input" v-model="path" placeholder="D:\Backup\settings.codexquota.json"></label>
    <button :disabled="busy" @click="run('export_settings')">{{ t("导出设置") }}</button>
    <button :disabled="busy" @click="run('import_settings')">{{ t("导入设置") }}</button>
    <button :disabled="busy" @click="run('export_diagnostics')">{{ t("导出脱敏诊断") }}</button>
    <button :disabled="busy" @click="run('validate_update_manifest')">{{t('检查更新清单格式')}}</button>
    <p>{{ t("设置扩展名：.codexquota.json；诊断扩展名：.codexquota-diagnostics.json。") }}</p>
    <p role="status">{{ result }}</p>
    <p>{{ t("自动更新尚未配置签名公钥，不会自动下载或安装。请使用已核验校验值的安装包升级。") }}</p>
  </section>
</template>
