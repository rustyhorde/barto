#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Build MUSL static binaries using Docker.

.PARAMETER Unstable
    Include unstable features.
#>
param(
    [switch]$Unstable
)

$releaseDir = "target/x86_64-unknown-linux-musl/release"
$bins = @("bartos", "bartoc", "barto-cli")

$dockerArgs = @(
    "run",
    "-v", "cargo-cache:/root/.cargo/registry",
    "-v", "${PWD}:/home/rust/src",
    "-v", "$($env:USERPROFILE)/.gitconfig:/root/.gitconfig:ro",
    "-e", "SQLX_OFFLINE=true",
    "--rm", "-t",
    "blackdex/rust-musl:x86_64-musl-nightly",
    "cargo", "build", "--release", "--locked"
)
if ($Unstable) { $dockerArgs += @("--features", "unstable") }

Write-Host "Building MUSL binaries with Docker..."
Write-Host "==> docker $($dockerArgs -join ' ')"
& docker @dockerArgs
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Command failed"
    exit 1
}

Write-Host "`nCopying binaries to home directory..."
foreach ($bin in $bins) {
    $src = Join-Path $releaseDir $bin
    $dst = Join-Path $env:USERPROFILE $bin
    Write-Host "==> Copy-Item $src $dst"
    Copy-Item -Path $src -Destination $dst -ErrorAction Stop
}

Write-Host "`n✓ MUSL binaries built and copied to:"
foreach ($bin in $bins) {
    Write-Host "  $(Join-Path $env:USERPROFILE $bin)"
}
