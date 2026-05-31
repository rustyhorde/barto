#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Run the full local CI pipeline.

.PARAMETER NoTest
    Skip tests and coverage.

.PARAMETER NoCoverage
    Skip coverage reports (keep tests).

.PARAMETER NoDocs
    Skip cargo doc.

.PARAMETER NoInstall
    Skip run_install.ps1.

.PARAMETER NoMusl
    Skip run_musl.ps1.

.PARAMETER Unstable
    Build MUSL with --features unstable.

.PARAMETER Clean
    Run cargo clean at end.
#>
param(
    [switch]$NoTest,
    [switch]$NoCoverage,
    [switch]$NoDocs,
    [switch]$NoInstall,
    [switch]$NoMusl,
    [switch]$Unstable,
    [switch]$Clean
)

function Invoke-Step {
    param([string]$Command)
    Write-Host "==> $Command"
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Command failed"
        exit 1
    }
}

Write-Host ""
Write-Host "=== Step 1: Format code ==="
Invoke-Step "cargo fmt --all"

Write-Host ""
Write-Host "=== Step 2: Check formatting ==="
Invoke-Step "cargo fmt --all -- --check"

Write-Host ""
Write-Host "=== Step 3: Clippy lint ==="
Invoke-Step "cargo matrix clippy --all-targets -- -Dwarnings"

Write-Host ""
Write-Host "=== Step 4: Build ==="
Invoke-Step "cargo matrix build"

if (-not $NoTest) {
    Write-Host ""
    Write-Host "=== Step 5: Tests ==="
    Invoke-Step "cargo nextest run -p libbarto"

    if (-not $NoDocs) {
        Write-Host ""
        Write-Host "=== Step 6: Documentation ==="
        Invoke-Step "cargo doc -p libbarto"
    }

    if (-not $NoCoverage) {
        Write-Host ""
        Write-Host "=== Step 7: Coverage ==="
        Invoke-Step "cargo llvm-cov nextest -F unstable --no-report --exclude barto-cli --exclude bartoc --exclude bartos --exclude xtask --workspace"

        Write-Host ""
        Write-Host "=== Step 8: Coverage report (LCOV) ==="
        Invoke-Step "cargo llvm-cov report --lcov --output-path lcov.info"

        Write-Host ""
        Write-Host "=== Step 9: Coverage report (HTML) ==="
        Invoke-Step "cargo llvm-cov report --html"
    }
} else {
    if (-not $NoDocs) {
        Write-Host ""
        Write-Host "=== Step 6: Documentation ==="
        Invoke-Step "cargo doc -p libbarto"
    }
}

if (-not $NoInstall) {
    Write-Host ""
    Write-Host "=== Step 10: Install ==="
    Write-Host "==> $PSScriptRoot\run_install.ps1"
    & "$PSScriptRoot\run_install.ps1"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Command failed"
        exit 1
    }
}

if (-not $NoMusl) {
    Write-Host ""
    Write-Host "=== Step 11: Build MUSL ==="
    $muslArgs = if ($Unstable) { @("-Unstable") } else { @() }
    Write-Host "==> $PSScriptRoot\run_musl.ps1 $($muslArgs -join ' ')"
    & "$PSScriptRoot\run_musl.ps1" @muslArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Command failed"
        exit 1
    }
}

if ($Clean) {
    Write-Host ""
    Write-Host "=== Step 12: Clean ==="
    Invoke-Step "cargo clean"
}

Write-Host ""
Write-Host "✓ CI pipeline complete"
