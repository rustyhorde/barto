#!/usr/bin/env pwsh

function Invoke-Step {
    param([string]$Command)
    Write-Host "==> $Command"
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Command failed"
        exit 1
    }
}

Invoke-Step "cargo install --force --locked -p bartos"
Invoke-Step "cargo install --force --locked -p bartoc"
Invoke-Step "cargo install --force --locked -p barto-cli"

Write-Host "✓ All binaries installed"
