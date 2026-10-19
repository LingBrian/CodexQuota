param([Parameter(Mandatory=$true)][string]$AppPath, [Parameter(Mandatory=$true)][string]$CliPath, [Parameter(Mandatory=$true)][string]$ReportDirectory, [switch]$TestBackground, [switch]$TestNotification)
$ErrorActionPreference = 'Stop'
$resolvedApp = (Resolve-Path -LiteralPath $AppPath).Path
$resolvedCli = (Resolve-Path -LiteralPath $CliPath).Path
if ([IO.Path]::GetFileName($resolvedCli) -ne 'codex-switch.exe') { throw 'CLI must be codex-switch.exe' }
$env:PATH = [IO.Path]::GetDirectoryName($resolvedCli) + ';' + $env:PATH
$env:CODEXQUOTA_DATA_DIR = [IO.Path]::GetFullPath($ReportDirectory)
$reportPath = Join-Path $env:CODEXQUOTA_DATA_DIR 'runtime-smoke.json'
if (Test-Path -LiteralPath $reportPath) { throw 'Use a new report directory to avoid stale evidence' }
$arguments=@('--smoke-test')
if($TestBackground){$arguments+='--test-background'}
if($TestNotification){$arguments+='--test-notification'}
$application = Start-Process -FilePath $resolvedApp -ArgumentList $arguments -PassThru -WindowStyle Hidden
$deadline=(Get-Date).AddSeconds($(if($TestBackground){110}else{60}))
while(-not $application.HasExited -and (Get-Date)-lt $deadline){$null=$application.WaitForExit(10000);$application.Refresh()}
if (-not $application.HasExited) { throw 'Application self-test did not finish in time' }
if (-not (Test-Path -LiteralPath $reportPath)) { throw "Application exited without producing a self-test report (exit $($application.ExitCode))" }
$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
$report | ConvertTo-Json -Depth 8
if (-not $report.passed -or $application.ExitCode -ne 0) { throw 'Application self-test failed' }
