// Mechanical replacements for dynamic display text. Internal action IDs stay Chinese.
import fs from 'node:fs'
const edits = {
  'src/App.vue': [[ '{{ item }}','{{ t(item) }}' ], ['{{ page }}','{{ t(page) }}'], ["{{ quota.data.account || '尚未选择账号' }}","{{ quota.data.account || t('尚未选择账号') }}"], ["{{ quota.loading ? '刷新中…' : '立即刷新' }}","{{ t(quota.loading ? '刷新中…' : '立即刷新') }}"], ['{{ status }}','{{ t(status) }}'], [": '暂无数据' }}", ": t('暂无数据') }}"]],
  'src/components/QuotaCard.vue': [["return '暂无数据'", "return t('暂无数据')"], [": '暂无数据' }", ": t('暂无数据') }"], ["return '等待重新查询'", "return t('等待重新查询')"], ["{{ label === '5h' ? '5 小时窗口' : '7 天窗口' }}", "{{ t(label === '5h' ? '5 小时窗口' : '7 天窗口') }}"], ["? '暂无数据' :", "? t('暂无数据') :"]],
  'src/components/AccountsView.vue': [["account.current ? '（当前）' : ''", "account.current ? t('（当前）') : ''"], ["{{ account.available ? '成功' : '需检查登录或网络' }}", "{{ t(account.available ? '成功' : '需检查登录或网络') }}"], ["{{ busy ? '处理中…' : '确认' }}", "{{ t(busy ? '处理中…' : '确认') }}"]],
  'src/components/OverlayWidget.vue': [["?'暂无数据':", "?t('暂无数据'):"]],
  'src/components/ReminderEditor.vue': [['{{testStatus}}','{{t(testStatus)}}']],
}
for (const [file, replacements] of Object.entries(edits)) {
  let source = fs.readFileSync(file,'utf8')
  for (const [from,to] of replacements) source=source.replaceAll(from,to)
  fs.writeFileSync(file,source)
}
