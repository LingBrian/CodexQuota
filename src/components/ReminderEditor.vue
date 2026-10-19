<script setup lang="ts">
import { t } from '../i18n'
import type { ReminderSettings } from '../types'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
const testStatus=ref('')
async function test(){try{await invoke('test_notification');testStatus.value='测试通知已提交给 Windows'}catch{testStatus.value='系统通知提交失败，请检查 Windows 通知设置'}}
defineProps<{value:ReminderSettings}>()
const emit=defineEmits<{save:[]}>()
function thresholds(text:string){return text.split(/[,，\s]+/).filter(Boolean).map(Number)}
</script>
<template>
  <article class="card"><div class="section-heading"><div><h2>{{t('提醒通道')}}</h2><p>{{t('额度变化时，通过 Windows 通知告知你。')}}</p></div><button @click="test">{{t('发送测试通知')}}</button></div><label>{{t('启用 Windows 原生通知')}}<input type="checkbox" v-model="value.enabled"></label><p v-if="testStatus" class="hint" role="status">{{t(testStatus)}}</p></article>
  <article class="card"><div class="section-heading"><div><h2>{{t('低额度阈值')}}</h2><p>{{t('同一周期、同一阈值只提醒一次。历史提醒状态保存在本机。')}}</p></div></div><div class="form-grid"><label>{{t('5h 阈值（逗号分隔）')}}<input :value="value.fiveHourThresholds.join(', ')" @change="value.fiveHourThresholds=thresholds(($event.target as HTMLInputElement).value)"></label><label>{{t('7d 阈值（逗号分隔）')}}<input :value="value.sevenDayThresholds.join(', ')" @change="value.sevenDayThresholds=thresholds(($event.target as HTMLInputElement).value)"></label></div></article>
  <article class="card"><div class="section-heading"><div><h2>{{t('安静时段')}}</h2><p>{{t('两个小时设为相同值可关闭免打扰。')}}</p></div></div><div class="form-grid"><label>{{t('免打扰开始小时')}}<input type="number" min="0" max="23" v-model.number="value.quietStart"></label><label>{{t('免打扰结束小时')}}<input type="number" min="0" max="23" v-model.number="value.quietEnd"></label></div><div class="toggle-grid"><label>{{t('额度恢复提醒')}}<input type="checkbox" v-model="value.recovery"></label><label>{{t('查询失败提醒')}}<input type="checkbox" v-model="value.queryFailure"></label><label>{{t('重置前 5 分钟提醒（仅使用可靠的重置时间）')}}<input type="checkbox" v-model="value.resetSoon"></label><label>{{t('提示音')}}<input type="checkbox" v-model="value.sound"></label></div><div class="form-footer"><button class="primary" @click="emit('save')">{{t('保存提醒设置')}}</button></div></article>
</template>
