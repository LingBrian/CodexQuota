import fs from 'node:fs'
import { spawnSync } from 'node:child_process'
import path from 'node:path'
const result=spawnSync('cargo',['metadata','--manifest-path','src-tauri/Cargo.toml','--format-version','1','--filter-platform','x86_64-pc-windows-msvc','--locked'],{encoding:'utf8',maxBuffer:32*1024*1024,windowsHide:true})
if(result.status!==0)throw new Error('Cargo dependency metadata could not be resolved')
const cargo=JSON.parse(result.stdout).packages.map(pkg=>({ecosystem:'cargo',name:pkg.name,version:pkg.version,license:pkg.license??'REVIEW_REQUIRED'}))
const cliResult=spawnSync('cargo',['metadata','--manifest-path','references/codex-switch-fe14af0d6b9317a464493816f8bd074eccfae785/Cargo.toml','--format-version','1','--filter-platform','x86_64-pc-windows-msvc','--locked'],{encoding:'utf8',maxBuffer:32*1024*1024,windowsHide:true})
if(cliResult.status!==0)throw new Error('Bundled CLI license metadata could not be resolved')
const cliPackages=JSON.parse(cliResult.stdout).packages
const cli=cliPackages.map(pkg=>({ecosystem:'cargo-cli',name:pkg.name,version:pkg.version,license:pkg.license??'REVIEW_REQUIRED'}))
const lock=JSON.parse(fs.readFileSync('package-lock.json','utf8'))
const npm=Object.entries(lock.packages).filter(([name])=>name).map(([location,pkg])=>({ecosystem:'npm',name:location.split('node_modules/').at(-1),version:pkg.version,license:pkg.license??'REVIEW_REQUIRED'}))
fs.mkdirSync('reports/licenses',{recursive:true})
const inventory=[...cargo,...cli,...npm].sort((a,b)=>a.ecosystem.localeCompare(b.ecosystem)||a.name.localeCompare(b.name))
fs.writeFileSync('reports/licenses/inventory.json',JSON.stringify(inventory,null,2))
const destination='src-tauri/licenses/dependencies'
fs.mkdirSync(destination,{recursive:true})
fs.writeFileSync(path.join(destination,'INVENTORY.json'),JSON.stringify(inventory,null,2))
let copied=0
function copyNotices(directory,ecosystem,name,version){
  if(!fs.existsSync(directory))return
  for(const entry of fs.readdirSync(directory,{withFileTypes:true})){
    if(entry.isFile()&&/^(LICEN[CS]E|COPYING|NOTICE)([._-]|$)/i.test(entry.name)){
      const target=path.join(destination,ecosystem,`${name.replaceAll('/','_')}-${version}`)
      fs.mkdirSync(target,{recursive:true});fs.copyFileSync(path.join(directory,entry.name),path.join(target,entry.name));copied++
    }
  }
}
for(const pkg of JSON.parse(result.stdout).packages)copyNotices(path.dirname(pkg.manifest_path),'cargo',pkg.name,pkg.version)
for(const pkg of cliPackages)copyNotices(path.dirname(pkg.manifest_path),'cargo-cli',pkg.name,pkg.version)
for(const [location,pkg] of Object.entries(lock.packages))if(location)copyNotices(location,'npm',location.split('node_modules/').at(-1),pkg.version)
console.log(`Recorded ${cargo.length+cli.length} Rust and ${npm.length} npm dependency licenses; ${inventory.filter(pkg=>pkg.license==='REVIEW_REQUIRED').length} require further review. Copied ${copied} license/notice texts.`)
