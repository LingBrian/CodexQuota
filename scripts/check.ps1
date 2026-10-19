$ErrorActionPreference = 'Stop'
function Invoke-Check([string]$Label, [scriptblock]$Action) {
    Write-Output $Label
    & $Action
    if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE" }
}
Invoke-Check 'Frontend tests' { npm test }
Invoke-Check 'Types' { npm run check }
Invoke-Check 'Frontend build' { npm run build }
Invoke-Check 'Rust formatting' { cargo fmt --manifest-path src-tauri/Cargo.toml -- --check }
Invoke-Check 'Rust tests' { cargo test --manifest-path src-tauri/Cargo.toml }
Invoke-Check 'Clippy' { cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings }
