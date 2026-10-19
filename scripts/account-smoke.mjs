// Integration test using synthetic credentials in isolated Codex directories.
// No real authentication file is read or switched.
import fs from 'node:fs'
import path from 'node:path'
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
const executable=path.resolve(process.argv[2]??'')
assert.equal(path.basename(executable),'codex-switch.exe')
const reports=path.resolve('reports')
fs.mkdirSync(reports,{recursive:true})
const root=fs.mkdtempSync(path.join(reports,'account-fixture-'))
const codex=path.join(root,'codex')
const store=path.join(root,'switch')
fs.mkdirSync(codex,{recursive:true})
fs.mkdirSync(store,{recursive:true})
fs.writeFileSync(path.join(store,'config.toml'),'[use]\nrestart_app_server = false\n')
function credentials(name){
  const payload={email:`${name}@example.invalid`,exp:Math.floor(Date.now()/1000)+86400,'https://api.openai.com/auth':{chatgpt_account_id:`fixture-${name}`,chatgpt_user_id:`fixture-user-${name}`,chatgpt_plan_type:'plus'}}
  const jwt=`eyJhbGciOiJub25lIn0.${Buffer.from(JSON.stringify(payload)).toString('base64url')}.fixture`
  return {auth_mode:'chatgpt',tokens:{id_token:jwt,access_token:jwt,refresh_token:'synthetic-not-a-real-token',account_id:`fixture-${name}`},last_refresh:new Date().toISOString()}
}
for(const name of ['alpha','beta']){
  fs.mkdirSync(path.join(store,'profiles',name),{recursive:true})
  fs.writeFileSync(path.join(store,'profiles',name,'auth.json'),JSON.stringify(credentials(name)))
}
fs.copyFileSync(path.join(store,'profiles','alpha','auth.json'),path.join(codex,'auth.json'))
fs.writeFileSync(path.join(store,'current'),'alpha')
const env={...process.env,CODEX_HOME:codex,CODEX_SWITCH_HOME:store}
function cli(...args){
  const result=spawnSync(executable,['--json',...args],{env,encoding:'utf8',timeout:30000,windowsHide:true})
  assert.equal(result.status,0,`CLI ${args[0]} must succeed in isolated fixture`)
  const output=JSON.parse(result.stdout)
  assert.equal(output.ok,true)
  return output
}
const beforeImport=fs.readFileSync(path.join(store,'profiles','alpha','auth.json'),'utf8')
const imported=cli('import',path.join(codex,'auth.json'),'import-check')
assert.equal(imported.action,'unchanged','existing identity must import without rotating credentials')
assert.equal(fs.readFileSync(path.join(store,'profiles','alpha','auth.json'),'utf8'),beforeImport)
cli('rename','beta','renamed-beta')
assert.equal(fs.existsSync(path.join(store,'profiles','beta')),false)
cli('use','renamed-beta')
assert.equal(fs.readFileSync(path.join(store,'current'),'utf8').trim(),'renamed-beta')
assert.equal(JSON.parse(fs.readFileSync(path.join(codex,'auth.json'),'utf8')).tokens.account_id,'fixture-beta')
cli('delete','alpha','--yes')
const archives=fs.readdirSync(path.join(store,'deleted-profiles'))
assert.equal(archives.length,1)
assert.equal(fs.existsSync(path.join(store,'deleted-profiles',archives[0],'auth.json')),true)
const result=spawnSync(executable,['--json','delete','renamed-beta','--yes'],{env,encoding:'utf8',timeout:30000,windowsHide:true})
assert.notEqual(result.status,0,'deleting the active account must be rejected')
assert.equal(fs.existsSync(path.join(store,'profiles','renamed-beta','auth.json')),true)
const report={syntheticCredentials:true,isolatedHome:true,existingIdentityImport:true,importDidNotRotateCredentials:true,renamed:true,switched:true,archived:true,activeDeleteRejected:true,passed:true}
fs.writeFileSync(path.join(root,'result.json'),JSON.stringify(report,null,2))
console.log(JSON.stringify(report,null,2))
console.log(`Report: ${path.relative(process.cwd(),path.join(root,'result.json'))}`)
