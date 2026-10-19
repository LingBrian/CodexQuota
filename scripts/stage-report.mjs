import fs from 'node:fs'
import path from 'node:path'
const reports=[]
for(const entry of fs.readdirSync('reports',{withFileTypes:true})){
  if(!entry.isDirectory()||!/^M\d+-/.test(entry.name))continue
  const file=path.join('reports',entry.name,'runtime-smoke.json')
  if(!fs.existsSync(file))continue
  const report=JSON.parse(fs.readFileSync(file,'utf8'))
  if(report.passed)reports.push({version:report.version,file:file.replaceAll('\\','/'),milestone:Number(report.version.split('.').at(-1))})
}
const newest=new Map()
for(const report of reports.sort((a,b)=>a.file.localeCompare(b.file)))newest.set(report.milestone,report)
const lines=['# 逐阶段桌面启动证据','','以下均为真实桌面进程自检报告，不是浏览器模拟额度。报告在本机保留；源码中仅记录路径，不上传账号快照。测试通过不代表物理硬件／跨系统矩阵已通过。','','| 阶段 | 版本 | 最近一次通过报告 |','|---|---|---|']
for(let milestone=1;milestone<=30;milestone++){
  const report=newest.get(milestone)
  lines.push(`| M${milestone} | v0.0.${milestone} | ${report?report.file:'未保留独立桌面报告；基础能力在后续全量回归中覆盖，不补造历史记录'} |`)
}
lines.push('','M28 的桌面自检不替代 24 小时长稳；M30 仍需安装／卸载、跨环境和稳定性门禁。Windows 10、物理第二显示器按用户要求保留待验收。','')
fs.writeFileSync('docs/STAGE_RUNS.md',lines.join('\n'))
console.log(`Recorded ${newest.size} milestone runtime reports.`)
