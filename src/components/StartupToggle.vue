<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { t } from '../i18n'
const enabled=ref(false), busy=ref(true), error=ref('')
onMounted(async()=>{try{enabled.value=await invoke('get_autostart')}catch(reason){error.value=String(reason)}finally{busy.value=false}})
async function toggle(event:Event){
  const previous=enabled.value;busy.value=true;error.value=''
  try{enabled.value=await invoke('set_autostart',{enabled:(event.target as HTMLInputElement).checked})}
  catch(reason){enabled.value=previous;(event.target as HTMLInputElement).checked=previous;error.value=String(reason)}
  finally{busy.value=false}
}
</script>
<template><label><input type="checkbox" :checked="enabled" :disabled="busy" @change="toggle">{{t('开机启动（当前用户，最小化到托盘）')}}</label><p v-if="error" role="alert">{{error}}</p></template>
