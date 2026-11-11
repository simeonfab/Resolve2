# Careers intake screenshots: every screen in full, empty, loading and error (where the screen
# has that state) at 1280 x 800 and 1920 x 1080, with the Careers canvases' sample in memory.
#
#   scripts\careers-shots.ps1 -Out <folder> [-Only <screen>] [-Sizes 1280x800,1920x1080]
#
# Standing rule: scripts never open the app on the real data folder. Every run uses a fresh
# sandbox under %TEMP%, and the script stops if it can't confirm that, or if the app refuses.
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Exe = "target\debug\resolve2.exe",
    [string]$Only = "",
    [string[]]$Sizes = @("1280x800", "1920x1080")
)
$ErrorActionPreference = "Stop"
$real = [IO.Path]::GetFullPath((Join-Path $env:APPDATA "Resolve2")).TrimEnd('\').ToLower()
$temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\').ToLower()
$sandbox = Join-Path ([IO.Path]::GetTempPath()) ("resolve2-careers-shots-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
$full = [IO.Path]::GetFullPath($sandbox).TrimEnd('\').ToLower()
if ($full -eq $real -or $full.StartsWith("$real\") -or -not $full.StartsWith("$temp\")) { throw "Refused: can't confirm $sandbox is a sandbox." }
New-Item -ItemType Directory -Force $sandbox, $Out | Out-Null
$Exe = (Resolve-Path $Exe).Path

# screen name -> (view, states it has, extra args)
$screens = [ordered]@{
    "01-name"        = @("careers-name", "full,empty", "")
    "02a-welcome"    = @("careers-welcome", "full,empty", "")
    "02b-bring"      = @("careers-bring", "full,empty,error", "")
    "02c-pull"       = @("careers-pull", "full,empty", "")
    "03-questions"   = @("careers-questions", "full,empty,loading,error", "")
    "04-confirm"     = @("careers-confirm", "full,empty,loading,error", "")
    "04b-edit"       = @("careers-edit", "full,empty", "")
    "05-outcome"     = @("careers-outcome", "full,empty,error", "")
    "06-next-move"   = @("careers-proposal-0", "full,empty,loading,error", "")
    "07-gates"       = @("careers-proposal-1", "full,empty,loading,error", "")
    "08-first-week"  = @("careers-proposal-2", "full,loading,error", "")
    "09-kit"         = @("careers-proposal-3", "full,empty,loading,error", "")
    "editor-cv"      = @("careers-editor", "full,empty,loading,error", "editor:CV templates")
    "editor-gates"   = @("careers-editor", "full,empty", "editor:Gates")
    "editor-target"  = @("careers-editor", "full", "editor:Target")
    "editor-evidence" = @("careers-editor", "full,empty", "editor:Evidence bank")
    "10-bridge"      = @("careers-bridge", "full", "")
    "11-reveal"      = @("careers-reveal", "full", "")
    "12-limit"       = @("careers-limit", "full", "")
}
$n = 0
foreach ($key in $screens.Keys) {
    if ($Only -and $key -notlike "*$Only*") { continue }
    $view, $states, $open = $screens[$key]
    foreach ($state in $states.Split(',')) {
        foreach ($size in $Sizes) {
            $data = Join-Path $sandbox "data"
            $a = @("--data-dir", $data, "--theme", "light", "--now", "2026-11-06T14:00", "--state", $state, "--size", $size, "--view", $view, "--screenshot", (Join-Path $Out "$key-$state-$size.png"))
            if ($open) { $a += @("--open", $open) }
            $q = $a | ForEach-Object { if ($_ -match '\s') { "`"$_`"" } else { $_ } }
            $p = Start-Process -FilePath $Exe -ArgumentList $q -PassThru -WindowStyle Hidden
            if (-not $p.WaitForExit(60000)) { $p.Kill(); throw "Timed out: $key $state $size" }
            if ($p.ExitCode -eq 2) { throw "The app refused the folder $data." }
            $n++
        }
    }
}
"$n screenshots in $Out"
