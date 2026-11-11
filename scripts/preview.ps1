# Build the release preview of the current branch and put it on the desktop (AGENTS.md rule 9).
#
#   powershell -NoProfile -File scripts\preview.ps1
#
# 1. A release build marked with the branch (RESOLVE2_PREVIEW): it only ever uses
#    %APPDATA%\Resolve2-Preview, refuses the real folder, and runs no sync, hub or server.
# 2. Installed as %LOCALAPPDATA%\Resolve2-Preview\Resolve2-Preview.exe, replacing the last one.
# 3. %APPDATA%\Resolve2-Preview: a fresh COPY of the real data (read only, never moved; backups
#    left out), plus the café sample workspace.
# 4. A "Resolve 2 Preview" shortcut on the desktop.
# It never writes to %APPDATA%\Resolve2, and stops if it can't confirm that.
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

$branch = (git rev-parse --abbrev-ref HEAD).Trim()
$version = ((Select-String -Path Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value)
$real = [IO.Path]::GetFullPath((Join-Path $env:APPDATA "Resolve2")).TrimEnd('\')
$data = [IO.Path]::GetFullPath((Join-Path $env:APPDATA "Resolve2-Preview")).TrimEnd('\')
$home2 = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA "Resolve2-Preview")).TrimEnd('\')
$exe = Join-Path $home2 "Resolve2-Preview.exe"

# the guard: the preview's folders are not the real one, nor inside it
foreach ($p in @($data, $home2)) {
    if ($p -ieq $real -or $p.ToLower().StartsWith($real.ToLower() + "\")) { throw "Refused: $p is the real data folder." }
    if (-not $p.EndsWith("\Resolve2-Preview")) { throw "Refused: can't confirm $p is the preview folder." }
}

# a running preview keeps its data folder busy: ask, never close someone's window
if (Get-Process -Name "Resolve2-Preview" -ErrorAction SilentlyContinue) {
    throw "Resolve 2 Preview is open. Close it, then run this again (its data is replaced each time)."
}

"1/4 release build of $branch ($version)…"
$env:RESOLVE2_PREVIEW = $branch
try {
    cargo build --release --target-dir target-preview 2>&1 | Select-String "^error" -Context 0,6
    if ($LASTEXITCODE -ne 0) { throw "The release build failed." }
} finally {
    Remove-Item Env:\RESOLVE2_PREVIEW
}

"2/4 installing $exe"
New-Item -ItemType Directory -Force $home2 | Out-Null
Copy-Item "target-preview\release\resolve2.exe" $exe -Force

"3/4 fresh copy of your data into $data (your real folder is only read)"
if (Test-Path $data) { Remove-Item -Recurse -Force $data }
New-Item -ItemType Directory -Force $data | Out-Null
if (Test-Path $real) {
    robocopy $real $data /E /XD backups /XF *.tmp* /R:1 /W:1 /NFL /NDL /NJH /NJS /NP | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "Copying your data failed (robocopy $LASTEXITCODE)." }
}
& $exe --seed-sample
if ($LASTEXITCODE -ne 0) { throw "Adding the café sample failed." }

"4/4 desktop shortcut"
$desk = [Environment]::GetFolderPath("Desktop")
$sh = (New-Object -ComObject WScript.Shell).CreateShortcut((Join-Path $desk "Resolve 2 Preview.lnk"))
$sh.TargetPath = $exe
$sh.WorkingDirectory = $home2
$sh.Description = "Resolve 2 Preview: $branch, $version (its own data, never your real workspaces)"
$sh.IconLocation = "$exe,0"
$sh.Save()
"Preview ready: $branch, $version. Desktop shortcut: Resolve 2 Preview."
