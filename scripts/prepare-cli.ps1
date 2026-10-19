$ErrorActionPreference = 'Stop'
$commit = 'fe14af0d6b9317a464493816f8bd074eccfae785'
$expectedHash = '1EDD196C0C10B41552CFC4545A81591492465D88BA3B535D0B8D5F304EFF8CA4'
$archive = Join-Path (Get-Location) 'references/codex-switch-fe14af0.zip'
New-Item -ItemType Directory -Path references -Force | Out-Null
if (-not (Test-Path -LiteralPath $archive)) { Invoke-WebRequest -Uri "https://codeload.github.com/xjoker/codex-switch/zip/$commit" -OutFile $archive -TimeoutSec 60 }
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expectedHash) { throw 'Upstream source archive checksum mismatch' }
$source = Join-Path (Get-Location) "references/codex-switch-$commit"
if (-not (Test-Path -LiteralPath $source)) { Expand-Archive -LiteralPath $archive -DestinationPath references }
$patchedLockHash = 'C3202A84525E3F14B017F1F19A7640B57DA5B3B7336A4CB83711478050FE33AF'
$lock = Join-Path $source 'Cargo.lock'
if ((Get-FileHash -LiteralPath $lock -Algorithm SHA256).Hash -ne $patchedLockHash) {
    & cargo update --manifest-path (Join-Path $source 'Cargo.toml') -p rustls --precise 0.23.46
    if ($LASTEXITCODE -ne 0) { throw 'rustls security patch failed' }
    & cargo update --manifest-path (Join-Path $source 'Cargo.toml') -p quinn-proto --precise 0.11.18
    if ($LASTEXITCODE -ne 0) { throw 'quinn-proto security patch failed' }
}
if ((Get-FileHash -LiteralPath $lock -Algorithm SHA256).Hash -ne $patchedLockHash) { throw 'Patched dependency lock checksum mismatch' }
& cargo build --manifest-path (Join-Path $source 'Cargo.toml') --locked --release --target-dir references/codex-switch-target
if ($LASTEXITCODE -ne 0) { throw 'Upstream CLI build failed' }
New-Item -ItemType Directory -Path src-tauri/binaries,src-tauri/licenses/codex-switch,src-tauri/licenses/upstream-codex -Force | Out-Null
Copy-Item -LiteralPath references/codex-switch-target/release/codex-switch.exe -Destination src-tauri/binaries/codex-switch-x86_64-pc-windows-msvc.exe -Force
Copy-Item -LiteralPath (Join-Path $source 'LICENSE') -Destination src-tauri/licenses/codex-switch/LICENSE -Force
Copy-Item -LiteralPath (Join-Path $source 'assets/upstream-codex/LICENSE') -Destination src-tauri/licenses/upstream-codex/LICENSE -Force
Copy-Item -LiteralPath (Join-Path $source 'assets/upstream-codex/NOTICE.md') -Destination src-tauri/licenses/upstream-codex/NOTICE.md -Force
Write-Output "Pinned CLI prepared: $commit"
