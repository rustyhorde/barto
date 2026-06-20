#!/usr/bin/env pwsh

$runTests     = $true
$runCoverage  = $true
$runDocs      = $true
$runInstall   = $false
$runMusl      = $false
$muslUnstable = $false
$runClean     = $false

foreach ($arg in $args) {
    switch ($arg) {
        { $_ -in @('--help', '-h') } {
            Write-Host "Usage: run_all.ps1 [OPTIONS]"
            Write-Host ""
            Write-Host "Runs the full barto CI pipeline locally."
            Write-Host ""
            Write-Host "Options:"
            Write-Host "  --no-test      Skip nextest and all coverage steps"
            Write-Host "  --no-coverage  Skip coverage steps only (lcov + html reports)"
            Write-Host "  --no-docs      Skip the documentation step"
            Write-Host "  --install      Run the cargo install step"
            Write-Host "  --musl         Run the MUSL Docker build step (stable)"
            Write-Host "  --unstable     Run the MUSL Docker build step with the unstable feature"
            Write-Host "  --clean        Run cargo clean after all steps complete"
            Write-Host "  --help, -h     Show this help message"
            Write-Host ""
            Write-Host "Steps (in order):"
            Write-Host "  1.  cargo fmt --all"
            Write-Host "  2.  cargo fmt --all -- --check"
            Write-Host "  3.  cargo matrix clippy --all-targets -- -Dwarnings"
            Write-Host "  4.  cargo matrix build"
            Write-Host "  5.  cargo matrix nextest run                          (skipped with --no-test)"
            Write-Host "  6.  cargo test -p libbarto --doc                      (skipped with --no-test)"
            Write-Host "  7.  cargo doc -p libbarto                             (skipped with --no-docs)"
            Write-Host "  8.  cargo matrix -c coverage llvm-cov nextest ...     (skipped with --no-test or --no-coverage)"
            Write-Host "  9.  cargo llvm-cov report --lcov ...                  (skipped with --no-test or --no-coverage)"
            Write-Host "  10. cargo llvm-cov report --html                     (skipped with --no-test or --no-coverage)"
            Write-Host "  11. run_install.ps1                                  (only with --install)"
            Write-Host "  12. run_musl.ps1                                     (only with --musl or --unstable; --unstable builds unstable)"
            Write-Host "  13. cargo clean                                      (only with --clean)"
            exit 0
        }
        '--no-test' {
            $runTests    = $false
            $runCoverage = $false
        }
        '--no-coverage' { $runCoverage = $false }
        '--no-docs'     { $runDocs     = $false }
        '--install'     { $runInstall  = $true  }
        '--musl'        { $runMusl      = $true  }
        '--unstable'    { $runMusl = $true; $muslUnstable = $true }
        '--clean'       { $runClean    = $true  }
        default {
            Write-Host "Unknown argument: $arg"
            Write-Host "Run 'run_all.ps1 --help' for usage."
            exit 1
        }
    }
}

function Invoke-Step([string]$Command) {
    Write-Host ""
    Write-Host "==> $Command"
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Command"
        exit 1
    }
}

function Invoke-Script {
    param([string]$Path, [string[]]$ScriptArgs = @())
    $display = if ($ScriptArgs.Count -gt 0) { "$Path $($ScriptArgs -join ' ')" } else { $Path }
    Write-Host ""
    Write-Host "==> $display"
    & $Path @ScriptArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $display"
        exit 1
    }
}

Invoke-Step 'cargo fmt --all'
Invoke-Step 'cargo fmt --all -- --check'
Invoke-Step 'cargo matrix clippy --all-targets -- -Dwarnings'
Invoke-Step 'cargo matrix build'

if ($runTests) {
    Invoke-Step 'cargo matrix nextest run'
    Invoke-Step 'cargo test -p libbarto --doc'
}

if ($runDocs) {
    Invoke-Step 'cargo doc -p libbarto'
}

if ($runCoverage) {
    Invoke-Step 'cargo matrix -c coverage llvm-cov nextest --no-report'
    Invoke-Step 'cargo llvm-cov report --lcov --output-path lcov.info'
    Invoke-Step 'cargo llvm-cov report --html'
}

if ($runInstall) {
    Invoke-Script "$PSScriptRoot\run_install.ps1"
}

if ($runMusl) {
    $muslArgs = if ($muslUnstable) { @('-Unstable') } else { @() }
    Invoke-Script "$PSScriptRoot\run_musl.ps1" $muslArgs
}

if ($runClean) {
    Invoke-Step 'cargo clean'
}

Write-Host ""
Write-Host "All steps completed successfully."
