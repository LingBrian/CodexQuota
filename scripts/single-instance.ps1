param([Parameter(Mandatory=$true)][string]$AppPath,[Parameter(Mandatory=$true)][string]$CliPath,[string]$SettingsPath)
$ErrorActionPreference='Stop'
$resolvedApp=(Resolve-Path -LiteralPath $AppPath).Path
$resolvedCli=(Resolve-Path -LiteralPath $CliPath).Path
$directory=Join-Path (Get-Location) ('reports/single-instance-'+(Get-Date -Format 'yyyyMMddTHHmmss'))
New-Item -ItemType Directory -Path $directory | Out-Null
if($SettingsPath){Copy-Item -LiteralPath $SettingsPath -Destination (Join-Path $directory 'settings.json')}
$env:CODEXQUOTA_DATA_DIR=$directory
$env:PATH=[IO.Path]::GetDirectoryName($resolvedCli)+';'+$env:PATH
$first=Start-Process -FilePath $resolvedApp -PassThru -WindowStyle Hidden
try {
    Start-Sleep -Seconds 3
    $first.Refresh()
    if($first.HasExited){throw 'First application instance failed to remain running'}
    $second=Start-Process -FilePath $resolvedApp -ArgumentList '--show' -PassThru -WindowStyle Hidden
    if(-not $second.WaitForExit(10000) -or $second.ExitCode -ne 0){throw 'Second instance was not rejected cleanly'}
    $first.Refresh()
    if($first.HasExited){throw 'Original instance unexpectedly exited'}
    $quit=Start-Process -FilePath $resolvedApp -ArgumentList '--quit' -PassThru -WindowStyle Hidden
    if(-not $quit.WaitForExit(10000)){throw 'Quit request did not return'}
    if(-not $first.WaitForExit(10000) -or $first.ExitCode -ne 0){throw 'Application did not exit cleanly'}
    $report=@{secondInstanceRejected=$true;originalInstancePreserved=$true;cleanExit=$true;passed=$true}
    $report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $directory 'result.json')
    $report | ConvertTo-Json
} finally {
    $first.Refresh()
    if(-not $first.HasExited){$quit=Start-Process -FilePath $resolvedApp -ArgumentList '--quit' -PassThru -WindowStyle Hidden;$null=$quit.WaitForExit(10000);$null=$first.WaitForExit(10000)}
}
