param(
    [Parameter(Mandatory=$true)][string]$AppPath,
    [Parameter(Mandatory=$true)][string]$CliPath,
    [ValidateRange(60,172800)][int]$DurationSeconds = 86400
)
$ErrorActionPreference = 'Stop'
$resolvedApp = (Resolve-Path -LiteralPath $AppPath).Path
$resolvedCli = (Resolve-Path -LiteralPath $CliPath).Path
$runDirectory = Join-Path (Get-Location) ('reports/soak-' + (Get-Date -Format 'yyyyMMddTHHmmss'))
New-Item -ItemType Directory -Path $runDirectory | Out-Null
$env:CODEXQUOTA_DATA_DIR = $runDirectory
$env:PATH = [IO.Path]::GetDirectoryName($resolvedCli) + ';' + $env:PATH
$process = Start-Process -FilePath $resolvedApp -ArgumentList '--soak-test',"--soak-duration-seconds=$DurationSeconds" -PassThru -WindowStyle Hidden
$started = Get-Date
$samples = 0
$maximumWorkingSet = 0L
$unexpectedExit = $false
$lastSampleAt = $started
$maximumSamplingGap = 0.0
$samplesPath = Join-Path $runDirectory 'samples.jsonl'
while (((Get-Date) - $started).TotalSeconds -lt ($DurationSeconds + 30)) {
    $sampleNow = Get-Date
    $maximumSamplingGap = [Math]::Max($maximumSamplingGap, ($sampleNow - $lastSampleAt).TotalSeconds)
    $lastSampleAt = $sampleNow
    $process.Refresh()
    if ($process.HasExited) {
        $unexpectedExit = (((Get-Date) - $started).TotalSeconds -lt ($DurationSeconds - 2)) -or $process.ExitCode -ne 0
        break
    }
    $maximumWorkingSet = [Math]::Max($maximumWorkingSet, $process.WorkingSet64)
    $sample = @{ elapsedSeconds = [int]((Get-Date) - $started).TotalSeconds; workingSetBytes = $process.WorkingSet64; cpuSeconds = $process.TotalProcessorTime.TotalSeconds }
    [IO.File]::AppendAllText($samplesPath, (($sample | ConvertTo-Json -Compress) + [Environment]::NewLine))
    $samples++
    Start-Sleep -Seconds ([Math]::Min(30,$DurationSeconds))
}
$process.Refresh()
$completed = $process.HasExited -and -not $unexpectedExit -and $process.ExitCode -eq 0
$continuousSampling = $maximumSamplingGap -le 120 -and $samples -ge [Math]::Floor($DurationSeconds / 30 * 0.9)
$report = @{ completed = $completed; requestedSeconds = $DurationSeconds; elapsedSeconds = [int]((Get-Date)-$started).TotalSeconds; samples = $samples; peakWorkingSetBytes = $maximumWorkingSet; metricScope = 'tauri-host-only'; maximumSamplingGapSeconds = $maximumSamplingGap; continuousSampling = $continuousSampling; qualifiesForM28 = ($completed -and $DurationSeconds -ge 86400 -and $continuousSampling) }
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $runDirectory 'result.json')
$report | ConvertTo-Json
if (-not $completed) { throw "Stability test incomplete; inspect $runDirectory" }
