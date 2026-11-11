# Careers Step 3 screenshots: Home, Projects, Tasks and Calendar in a Careers workspace, in full,
# empty, loading and error, at 1280 x 800 and 1920 x 1080 (the Careers sample, in memory).
#
#   scripts\careers-daily-shots.ps1 -Out <folder>
#
# Standing rule: a fresh sandbox under %TEMP%; the script stops if it can't confirm that.
param([Parameter(Mandatory = $true)][string]$Out, [string]$Exe = "target\debug\resolve2.exe")
$ErrorActionPreference = "Stop"
$real = [IO.Path]::GetFullPath((Join-Path $env:APPDATA "Resolve2")).TrimEnd('\').ToLower()
$temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\').ToLower()
$sandbox = Join-Path ([IO.Path]::GetTempPath()) ("resolve2-careers-daily-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
$full = [IO.Path]::GetFullPath($sandbox).TrimEnd('\').ToLower()
if ($full -eq $real -or $full.StartsWith("$real\") -or -not $full.StartsWith("$temp\")) { throw "Refused: can't confirm $sandbox is a sandbox." }
New-Item -ItemType Directory -Force $sandbox, $Out | Out-Null
$Exe = (Resolve-Path $Exe).Path
$n = 0
foreach ($v in @("home", "projects", "tasks@p", "calendar")) {
    foreach ($state in @("full", "empty", "loading", "error")) {
        foreach ($size in @("1280x800", "1920x1080")) {
            $name = ($v -replace '@p', '') + "-$state-$size.png"
            $a = @("--data-dir", (Join-Path $sandbox "data"), "--theme", "light", "--state", $state, "--careers", "--size", $size, "--view", $v, "--screenshot", (Join-Path $Out $name))
            $p = Start-Process -FilePath $Exe -ArgumentList $a -PassThru -WindowStyle Hidden
            if (-not $p.WaitForExit(60000)) { $p.Kill(); throw "Timed out: $name" }
            if ($p.ExitCode -eq 2) { throw "The app refused the folder." }
            $n++
        }
    }
}
"$n screenshots in $Out"
