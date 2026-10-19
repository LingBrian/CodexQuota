param([ValidateRange(1,30)][int]$Milestone, [Parameter(Mandatory=$true)][string]$CliPath)
$ErrorActionPreference = 'Stop'
function Run-StageCommand([string]$Name, [scriptblock]$Command) {
    Write-Output $Name
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "$Name failed ($LASTEXITCODE)" }
}
Run-StageCommand "Version M$Milestone" { node scripts/version.mjs "0.0.$Milestone" }
Run-StageCommand 'Quality gate' { & ./scripts/check.ps1 }
Run-StageCommand 'Desktop build' { npx tauri build --debug --no-bundle }
$reportDirectory = Join-Path 'reports' ("M{0}-{1}" -f $Milestone, (Get-Date -Format 'yyyyMMddTHHmmss'))
& ./scripts/smoke.ps1 -AppPath ./src-tauri/target/debug/codexquota.exe -CliPath $CliPath -ReportDirectory $reportDirectory
Write-Output "M$Milestone desktop runtime verified: $reportDirectory/runtime-smoke.json"
