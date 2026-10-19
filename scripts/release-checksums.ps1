param([string]$BundleDirectory = 'src-tauri/target/release/bundle', [string]$Version = '')
$ErrorActionPreference = 'Stop'
if (-not $Version) { $Version = (Get-Content -LiteralPath 'src-tauri/tauri.conf.json' -Raw | ConvertFrom-Json).version }
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'Invalid release version' }
$files = @(Get-ChildItem -LiteralPath $BundleDirectory -Recurse -File | Where-Object { $_.Extension -in '.msi','.exe' -and $_.Name -like "CodexQuota_${Version}_*" })
if ($files.Count -eq 0) { throw 'No installers were produced' }
$lines = foreach ($file in $files) { $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); "$hash *$($file.Name)" }
[IO.File]::WriteAllLines((Join-Path (Resolve-Path $BundleDirectory).Path 'SHA256SUMS.txt'),$lines)
$lines
