# Invoke with pwsh -NoProfile -File .a4-shim/run.ps1.
param([string]$LogName = 'test-shimmed.log')
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
Set-Location $repoRoot
rustc "$PSScriptRoot/shim.rs" -o "$PSScriptRoot/claude.exe"
if ($LASTEXITCODE -ne 0) { throw 'Cannot compile no-spawn shim' }
Copy-Item "$PSScriptRoot/claude.exe" "$PSScriptRoot/codex.exe" -Force
$env:PATH = "$PSScriptRoot;$repoRoot/.a4-receipts/protoc/bin;$env:PATH"
$env:CARGO_BUILD_JOBS = '2'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
foreach ($name in @('claude', 'codex')) {
    where.exe $name
    $resolved = (Get-Command $name -CommandType Application | Select-Object -First 1).Source
    if ($resolved -ne "$PSScriptRoot\$name.exe") {
        throw "Unexpected executable: $resolved"
    }
    & $resolved --version
    if ($LASTEXITCODE -ne 1) { throw "Unexpected $name shim exit" }
    Write-Output "$name --version: exit 1"
}
cargo test --workspace --locked --no-fail-fast *> "$repoRoot/.a4-receipts/$LogName"
$suiteExit = $LASTEXITCODE
Set-Content "$repoRoot/.a4-receipts/$LogName.exit" $suiteExit
exit $suiteExit
