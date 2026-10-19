import fs from 'node:fs'
import path from 'node:path'
const ignored=new Set(['.git','node_modules','references','reports','release','target','binaries','tools'])
const patterns=[['github-token',/gh[pousr]_[A-Za-z0-9]{30,}/],['api-key',/\bsk-[A-Za-z0-9_-]{24,}\b/],['private-key',/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/],['jwt-literal',/eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{16,}/]]
const findings=[]
let scanned=0
function walk(directory){
  for(const entry of fs.readdirSync(directory,{withFileTypes:true})){
    if(ignored.has(entry.name)||entry.isSymbolicLink())continue
    const file=path.join(directory,entry.name)
    if(entry.isDirectory()){walk(file);continue}
    if(/\.(exe|dll|png|ico|icns|jpg|jpeg|zip|msi|webp)$/i.test(file))continue
    const bytes=fs.readFileSync(file);if(bytes.includes(0))continue
    scanned++
    for(const [kind,pattern] of patterns)if(pattern.test(bytes.toString('utf8')))findings.push({file,kind})
  }
}
walk('.')
fs.mkdirSync('reports/security',{recursive:true})
const report={scanned,findings,passed:findings.length===0,scope:'source-text-pattern-scan; not a proof of absence'}
fs.writeFileSync('reports/security/source-scan.json',JSON.stringify(report,null,2))
console.log(JSON.stringify(report,null,2))
if(findings.length)process.exitCode=1
