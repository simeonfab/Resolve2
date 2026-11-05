# Regenerates docs/THIRD_PARTY_CRATES.md from Cargo.lock. Run from the repo root: powershell -File scripts/licenses.ps1
$m = cargo metadata --format-version 1 --locked | ConvertFrom-Json
$ids = $m.resolve.nodes.id
$pk = $m.packages | Where-Object { $ids -contains $_.id -and $_.name -ne 'resolve2' } | Sort-Object name, version
$lines = @('# Third-party Rust crates (generated)', '', "Generated from ``cargo metadata --locked`` on $(Get-Date -Format 'yyyy-MM-dd').", '', '| Crate | Version | Licence |', '|---|---|---|')
$lines += $pk | ForEach-Object { "| $($_.name) | $($_.version) | $($_.license) |" }
$lines | Set-Content -Encoding utf8 docs\THIRD_PARTY_CRATES.md
"$($pk.Count) crates written"
