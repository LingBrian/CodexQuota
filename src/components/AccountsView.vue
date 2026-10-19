<script setup lang="ts">
import { t } from '../i18n'
import { computed, ref, onMounted, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useQuotaStore } from '../stores/quota'
import { quotaLabel, remainingPercent } from '../utils/quota'
import type { QuotaSnapshot } from '../types'
const quota = useQuotaStore()
const sort = ref('name'), alias = ref(''), path = ref(''), newAlias = ref('')
const error = ref(''), busy = ref(false)
const preview = ref<{ticketId:string;alias:string;sizeBytes:number}>()
type Action = {kind:'switch'|'delete';alias:string} | {kind:'rename';alias:string;newAlias:string} | {kind:'import';ticketId:string} | {kind:'selectBest'}
const pending = ref<Action>()
const dialog = ref<HTMLDialogElement>()
let previousFocus:HTMLElement|null = null
watch(pending, async action => {
  await nextTick()
  if (action && dialog.value && !dialog.value.open) {
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
    dialog.value.showModal()
  } else if (!action && dialog.value?.open) { dialog.value.close(); previousFocus?.focus() }
})
function cancel(event:Event) { if(busy.value) event.preventDefault(); else pending.value=undefined }
const deleted = ref<{id:string;alias:string}[]>([])
const restoreId = ref(''), restoreAlias = ref('')
const rows = computed(() => [...(quota.data.accounts ?? [])].sort((a,b) => sort.value === 'name' ? a.alias.localeCompare(b.alias) : (remainingPercent(sort.value === 'sevenDay' ? b.sevenDay : b.fiveHour) ?? -1) - (remainingPercent(sort.value === 'sevenDay' ? a.sevenDay : a.fiveHour) ?? -1) || a.alias.localeCompare(b.alias)))
async function loadDeleted() { try { deleted.value=await invoke('deleted_accounts') } catch { error.value='归档列表读取失败，可手动导入归档中的认证文件' } }
const localAuth = ref<{path:string;suggestedAlias:string}[]>([])
const scanning = ref(false)
async function scanLocal() {
  scanning.value=true
  try { localAuth.value=await invoke('discover_local_auth') }
  catch { error.value='本机认证文件扫描失败，请检查目录权限' }
  finally { scanning.value=false }
}
function chooseLocal(item:{path:string;suggestedAlias:string}) {
  path.value=item.path
  let candidate=item.suggestedAlias
  let suffix=2
  while(rows.value.some(row=>row.alias===candidate)) candidate=`${item.suggestedAlias}-${suffix++}`
  alias.value=candidate
  preview.value=undefined
}
onMounted(() => { void loadDeleted(); void scanLocal() })
async function prepareRestore() {
  busy.value=true;error.value=''
  try { preview.value=await invoke('preview_restore',{id:restoreId.value,alias:restoreAlias.value});if(preview.value)pending.value={kind:'import',ticketId:preview.value.ticketId} }
  catch(reason){error.value=String(reason)}finally{busy.value=false}
}
async function inspect() {
  error.value='';preview.value=undefined;busy.value=true
  try{preview.value=await invoke('preview_import',{path:path.value,alias:alias.value})}catch(reason){error.value=String(reason)}finally{busy.value=false}
}
async function execute() {
  if(!pending.value)return
  busy.value=true;error.value=''
  try{const action=pending.value.kind==='rename'?{...pending.value,newAlias:newAlias.value}:pending.value;quota.data=await invoke<QuotaSnapshot>('account_action',{action,confirmed:true});pending.value=undefined;preview.value=undefined}
  catch(reason){error.value=String(reason);if(pending.value?.kind==='import'){pending.value=undefined;preview.value=undefined}}finally{busy.value=false;await loadDeleted()}
}
</script>
<template>
  <article class="card"><div class="section-heading"><div><h2>{{t('本地账号')}} <span class="count-badge">{{rows.length}}</span></h2><p>{{t('切换前先确认，当前账号不会被删除。')}}</p></div></div><p v-if="error" role="alert" class="error-banner">{{error}}</p>
    <div class="accounts-toolbar"><label>{{t('排序')}}<select v-model="sort"><option value="name">{{t('名称')}}</option><option value="quota">{{t('5h 剩余额度')}}</option><option value="sevenDay">{{t('7d 剩余额度')}}</option></select></label><button :disabled="busy" @click="pending={kind:'selectBest'}">{{t('自动选择可用账号')}}</button></div>
    <p v-if="!rows.length" class="empty-state">{{t('暂无已保存账号，可先导入已有的 Codex 认证文件。')}}</p>
    <div v-else class="table-scroll"><table><thead><tr><th>{{t('名称')}}</th><th>5h</th><th>7d</th><th>{{t('查询状态')}}</th><th>{{t('操作')}}</th></tr></thead><tbody><tr v-for="account in rows" :key="account.alias" data-account><td><span class="table-avatar">{{account.alias.slice(0,1).toUpperCase()}}</span>{{account.alias}} <span v-if="account.current" class="account-badge">{{t('（当前）')}}</span></td><td class="tabular">{{quotaLabel(account.fiveHour)}}</td><td class="tabular">{{quotaLabel(account.sevenDay)}}</td><td><span class="account-status" :class="{available:account.available}"><i></i>{{t(account.available?'成功':'需检查登录或网络')}}</span></td><td class="account-actions"><button :disabled="busy||account.current" @click="pending={kind:'switch',alias:account.alias}">{{t('切换')}}</button><button :disabled="busy" @click="newAlias=account.alias;pending={kind:'rename',alias:account.alias,newAlias:account.alias}">{{t('重命名')}}</button><button :disabled="busy||account.current" @click="pending={kind:'delete',alias:account.alias}">{{t('删除')}}</button></td></tr></tbody></table></div>
  </article>
  <article class="card"><div class="section-heading"><div><h2>{{t('导入已有账号')}}</h2><p>{{t('选择本机已有的 JSON 认证文件。预览仅检查结构与冲突；确认后由 codex-switch 联网验证并保存。')}}</p></div><button :disabled="busy||scanning" @click="scanLocal">{{scanning?'扫描中…':'重新扫描本机'}}</button></div>
    <p v-if="!scanning&&!localAuth.length" class="empty-state">未发现文件形式的 ChatGPT 登录信息。请先在 Codex 登录，或填写已有 auth.json 路径。</p>
    <div v-for="item in localAuth" :key="item.path" class="archive-row"><span class="auth-path">{{item.path}}</span><button :disabled="busy" @click="chooseLocal(item)">选择此账号</button></div>
    <div class="import-grid"><label>{{t('文件路径')}}<input v-model="path" placeholder="C:\…\auth.json（也支持所在目录）" autocomplete="off" @input="preview=undefined"></label><label>{{t('账号别名')}}<input v-model="alias" placeholder="work-account" @input="preview=undefined"></label></div><div class="form-footer"><button :disabled="busy||!path||!alias" @click="inspect">{{t('预览导入')}}</button></div><div v-if="preview" class="import-preview"><p>{{t('准备导入：')}}{{preview.alias}}（{{preview.sizeBytes}}{{t('字节）')}}</p><button :disabled="busy" @click="pending={kind:'import',ticketId:preview.ticketId}">{{t('确认导入…')}}</button></div></article>
  <article class="card"><div class="section-heading"><div><h2>{{t('可恢复归档')}}</h2><p>{{t('删除的账号会保留恢复点，恢复时再次验证。')}}</p></div></div><p v-if="!deleted.length" class="empty-state">{{t('暂无归档账号')}}</p><div v-for="account in deleted" :key="account.id" class="archive-row"><span>{{account.alias}}</span><button @click="restoreId=account.id;restoreAlias=account.alias.slice(0,54)+'-restored'">{{t('恢复…')}}</button></div><div v-if="restoreId"><label>{{t('恢复后的别名')}}<input v-model="restoreAlias"></label><button :disabled="busy" @click="prepareRestore">{{t('预览恢复')}}</button></div></article>
  <dialog ref="dialog" @cancel="cancel" :aria-label="t('确认账号操作')"><template v-if="pending"><h2>{{t('确认账号操作')}}</h2><p v-if="pending.kind==='delete'">{{t('账号将移入可恢复归档。当前账号不可删除。')}}</p><p v-else-if="pending.kind==='import'">{{t('导入会联网验证登录状态，可能轮换过期凭据；请等待完成后再重试。')}}</p><p v-else-if="pending.kind==='rename'">{{t('修改账号别名。')}}<input v-model="newAlias" :aria-label="t('新账号别名')"></p><p v-else>{{t('切换将影响后续 Codex 会话，上游可能重启 app-server 服务。请确认运行中的工作可以继续。')}}</p><div class="form-footer"><button autofocus :disabled="busy" @click="pending=undefined">{{t('取消')}}</button><button :disabled="busy" class="primary" @click="execute">{{t(busy?'处理中…':'确认')}}</button></div></template></dialog>
</template>
<style scoped>
.auth-path { overflow-wrap: anywhere; min-width: 0; }
dialog:has(.error-banner) { max-height: 90vh; overflow: auto; }
</style>
