<script setup lang="ts">
import { t } from '../i18n'
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppSettings } from '../types'
const props = defineProps<{value:AppSettings}>()
const emit = defineEmits<{save:[]}>()
const monitors = ref<{index:number;name:string;scale:number}[]>([])
const presets = [
  {id:'capsule', name:'轻巧胶囊', note:'柔和圆角，双行额度轻盈呈现', progress:true, width:420, height:48, gap:8, cornerRadius:22, background:'#19221f', textColor:'#f2fff9', backgroundOpacity:0.96},
  {id:'bars', name:'双条进度', note:'清晰分组，时间与进度一目了然', progress:true, width:420, height:48, gap:12, cornerRadius:8, background:'#1b1e23', textColor:'#f4f7fb', backgroundOpacity:0.94},
  {id:'text', name:'透明文字', note:'无底色融入桌面，保留双色进度', progress:true, width:420, height:48, gap:10, cornerRadius:0, background:'#111827', textColor:'#ffffff', backgroundOpacity:0},
  {id:'light-glass', name:'亮色透明', note:'柔和浅色玻璃，适合亮色任务栏', progress:true, width:420, height:48, gap:10, cornerRadius:16, background:'#edf4f8', textColor:'#17212b', backgroundOpacity:0.78},
  {id:'dark-glass', name:'暗色透明', note:'轻透深色玻璃，降低桌面干扰', progress:true, width:420, height:48, gap:10, cornerRadius:16, background:'#111923', textColor:'#f4f7fb', backgroundOpacity:0.72},
  {id:'light-solid', name:'亮色不透明', note:'干净浅底和清晰深色文字', progress:true, width:420, height:48, gap:12, cornerRadius:9, background:'#f8fafc', textColor:'#202a35', backgroundOpacity:1},
  {id:'dark-solid', name:'暗色不透明', note:'稳重深底和柔和亮色文字', progress:true, width:420, height:48, gap:12, cornerRadius:9, background:'#171a20', textColor:'#f4f7fb', backgroundOpacity:1},
]
function preset(id:string) {
  const selected = presets.find(item => item.id === id)
  if (!selected || !props.value.overlay) return
  Object.assign(props.value.overlay, {displayStyle:selected.id, cornerRadius:selected.cornerRadius, progress:selected.progress, width:selected.width, height:selected.height, gap:selected.gap, background:selected.background, backgroundOpacity:selected.backgroundOpacity, showFive:true, showSeven:true})
  Object.assign(props.value, {overlayTextColor:selected.textColor, overlayFontSize:13})
  emit('save')
}
onMounted(async()=>{try{monitors.value=await invoke('list_monitors')}catch{/* Primary-monitor fallback remains available. */}})
</script>
<template>
  <template v-if="value.overlay">
    <article class="card"><div class="section-heading"><div><h2>{{t('选择你的显示风格')}}</h2><p>{{t('一键套用，再按你的喜好微调。')}}</p></div></div><div class="preset-grid"><button v-for="item in presets" :key="item.id" class="preset-option" :data-preset="item.id" @click="preset(item.id)"><span class="preset-art" :class="item.id"><span class="preset-row five"><i></i><em>5h</em><span class="preset-track"><u></u></span><b>68%</b><time>10/16 22:32</time></span><span class="preset-row seven"><i></i><em>7d</em><span class="preset-track"><u></u></span><b>42%</b><time>10/22 22:11</time></span></span><strong>{{t(item.name)}}</strong><small>{{t(item.note)}}</small></button></div></article>
    <article class="card" @change="emit('save')"><div class="section-heading"><div><h2>{{t('显示内容')}}</h2><p>双行并列显示 5h、7d 剩余额度、进度和重置时间。</p></div></div><div class="toggle-grid"><label>{{t('显示 5h')}}<input type="checkbox" v-model="value.overlay.showFive"></label><label>{{t('显示 7d')}}<input type="checkbox" v-model="value.overlay.showSeven"></label><label>{{t('显示进度条')}}<input type="checkbox" v-model="value.overlay.progress"></label><label>{{t('交换显示顺序')}}<input type="checkbox" v-model="value.overlay.reverse"></label><label>{{t('显示已使用比例')}}<input type="checkbox" v-model="value.overlay.showUsed"></label></div></article>
    <article class="card" @change="emit('save')"><div class="section-heading"><div><h2>{{t('色彩与排版')}}</h2><p>{{t('设置背景、低额度颜色与文字间距。')}}</p></div></div><div class="form-grid"><label>{{t('背景色')}}<input type="color" v-model="value.overlay.background"></label><label>{{t('低额度颜色')}}<input type="color" v-model="value.overlay.lowColor"></label><label>{{t('背景透明度')}}<input type="range" min="0" max="1" step="0.05" v-model.number="value.overlay.backgroundOpacity"></label><label>{{t('粗细')}}<input type="number" min="100" max="900" step="100" v-model.number="value.overlay.fontWeight"></label><label>{{t('间距')}}<input type="number" min="0" max="100" v-model.number="value.overlay.gap"></label></div></article>
    <article class="card" @change="emit('save')"><div class="section-heading"><div><h2>{{t('位置与交互')}}</h2><p>修改后自动保存。任务栏模式按任务栏高度缩放；浮窗模式可调整横纵坐标。</p></div></div><div class="form-grid">
      <label>显示位置<select v-model="value.overlay.placement" data-placement><option value="taskbar">任务栏内</option><option value="desktop">桌面浮窗</option></select></label>
      <label>{{t('显示器')}}<select v-model="value.overlay.monitor"><option :value="undefined">{{t('自动选择主显示器')}}</option><option v-for="monitor in monitors" :key="monitor.index" :value="monitor.index">{{monitor.name}}（{{monitor.scale*100}}%）</option></select></label>
      <label v-if="value.overlay.placement !== 'desktop'">任务栏沿栏偏移<input data-taskbar-offset type="number" min="-500" max="10000" step="10" v-model.number="value.overlay.edgeOffset"></label>
      <template v-else><label>距桌面左侧<input data-offset-x type="number" min="0" max="20000" v-model.number="value.overlay.offsetX"></label><label>距桌面顶部<input data-offset-y type="number" min="0" max="20000" v-model.number="value.overlay.offsetY"></label></template>
      <label>{{t('宽度')}}<input type="number" min="120" max="1000" v-model.number="value.overlay.width"></label><label>{{t('高度')}}<input type="number" min="24" max="160" v-model.number="value.overlay.height"></label></div><label>{{t('鼠标穿透')}}<input type="checkbox" v-model="value.overlay.clickThrough"></label><p class="hint">任务栏内使用独立覆盖窗口，不占用 Windows 原生任务栏布局槽位。可调整沿栏偏移避开应用按钮；开启穿透后点击将交给下方任务栏。</p></article>
  </template>
</template>
