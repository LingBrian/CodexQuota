import fs from 'node:fs'
const version = process.argv[2]
if (!/^0\.0\.(?:0|[1-9][0-9]{0,3})$/.test(version ?? '')) throw new Error('Version must use the 0.0.N sequence')
for (const file of ['package.json', 'src-tauri/tauri.conf.json']) {
  const value = JSON.parse(fs.readFileSync(file, 'utf8'))
  value.version = version
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n')
}
const lock = JSON.parse(fs.readFileSync('package-lock.json', 'utf8'))
lock.version = version
lock.packages[''].version = version
fs.writeFileSync('package-lock.json', JSON.stringify(lock, null, 2) + '\n')
for (const file of ['src-tauri/Cargo.toml', 'src-tauri/Cargo.lock']) {
  const value = fs.readFileSync(file, 'utf8')
  const expression = file.endsWith('.toml') ? /^(version = ")[^"]+/m : /(name = "codexquota"\r?\nversion = ")[^"]+/
  fs.writeFileSync(file, value.replace(expression, `$1${version}`))
}
console.log(`All application version metadata updated to v${version}`)
