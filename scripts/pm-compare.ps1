# PM comparison: the same PM screens from two builds (say main's and a branch's), on two copies of
# the sample data, same clock, each screen taken twice. A screen is unchanged when the two builds
# give a byte-identical picture (animated screens vary run to run within one build).
#
#   scripts\pm-compare.ps1 -MainExe <main build's resolve2.exe> [-BranchExe target\debug\resolve2.exe]
#
# Standing rule: scripts never open the app on the real data folder (%APPDATA%\Resolve2). Every run
# uses a fresh sandbox folder under %TEMP%, and the script stops if it can't confirm that, or if
# the app refuses the folder (exit 2).
param(
    [Parameter(Mandatory = $true)][string]$MainExe,
    [string]$BranchExe = "target\debug\resolve2.exe",
    [string]$Sample = "shots\p1\data",
    [int]$Runs = 2
)
$ErrorActionPreference = "Stop"

$real = [IO.Path]::GetFullPath((Join-Path $env:APPDATA "Resolve2")).TrimEnd('\').ToLower()
$sandbox = Join-Path ([IO.Path]::GetTempPath()) ("resolve2-pm-compare-" + (Get-Date -Format "yyyyMMdd-HHmmss"))

function Assert-Sandbox([string]$dir) {
    $full = [IO.Path]::GetFullPath($dir).TrimEnd('\').ToLower()
    $temp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\').ToLower()
    if ($full -eq $real -or $full.StartsWith("$real\")) { throw "Refused: $dir is the real data folder." }
    if (-not $full.StartsWith("$temp\")) { throw "Refused: can't confirm $dir is a sandbox (it isn't under $temp)." }
}

function Shot([string]$exe, [string]$data, [string[]]$rest) {
    Assert-Sandbox $data
    $all = @("--data-dir", $data) + $rest
    $q = $all | ForEach-Object { if ($_ -match '\s') { "`"$_`"" } else { $_ } }
    $p = Start-Process -FilePath $exe -ArgumentList $q -PassThru -WindowStyle Hidden
    if (-not $p.WaitForExit(60000)) { $p.Kill(); throw "Timed out: $($all -join ' ')" }
    if ($p.ExitCode -eq 2) { throw "The app refused the folder $data." }
}

$MainExe = (Resolve-Path $MainExe).Path
$BranchExe = (Resolve-Path $BranchExe).Path
$views = @("home", "projects", "calendar", "overview@p", "roadmap@p", "tasks", "frameworks@p", "knowledge@p", "team@p", "handoffs@p", "settings", "help", "intake-questions", "intake-confirm", "intake-proposal")
$hashes = @{}
foreach ($side in @(@{ n = "main"; exe = $MainExe }, @{ n = "branch"; exe = $BranchExe })) {
    $out = Join-Path $sandbox $side.n
    $data = Join-Path $out "data"
    New-Item -ItemType Directory -Force $out | Out-Null
    Assert-Sandbox $data
    Copy-Item -Recurse $Sample $data
    for ($r = 1; $r -le $Runs; $r++) {
        foreach ($size in @("1280x800", "1920x1080")) {
            foreach ($v in $views) {
                $name = ($v -replace '@p', '') + "-$size"
                $png = Join-Path $out "$name-run$r.png"
                Shot $side.exe $data @("--theme", "light", "--now", "2026-11-06T14:00", "--state", "full", "--size", $size, "--view", $v, "--screenshot", $png)
                if (-not (Test-Path $png)) { throw "No picture for $name ($($side.n))." }
                $hashes["$name|$($side.n)"] += @((Get-FileHash $png).Hash.Substring(0, 8))
            }
        }
    }
}
$same = 0
$names = $hashes.Keys | ForEach-Object { $_.Split('|')[0] } | Sort-Object -Unique
foreach ($k in $names) {
    $m = $hashes["$k|main"] | Select-Object -Unique
    $b = $hashes["$k|branch"] | Select-Object -Unique
    $match = @($b | Where-Object { $m -contains $_ }).Count -gt 0
    if ($match) { $same++ }
    "| $k | $($m -join ' ') | $($b -join ' ') | $(if ($match) { 'same picture' } else { 'no byte match' }) |"
}
"$same of $($names.Count) screens give a byte-identical picture from both builds. Pictures: $sandbox"
