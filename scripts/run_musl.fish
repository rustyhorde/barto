#!/usr/bin/env fish

set features ""
set help false

for arg in $argv
    if test "$arg" = "--unstable"
        set features "--features unstable"
    else if test "$arg" = "--help"
        set help true
    end
end

if test $help = true
    echo "Usage: run_musl.fish [OPTIONS]"
    echo ""
    echo "Build MUSL static binaries using Docker."
    echo ""
    echo "Options:"
    echo "  --unstable  Include unstable features"
    echo "  --help      Show this help message"
    exit 0
end

function run_step
    echo "==> $argv"
    eval $argv
    if test $status -ne 0
        echo "Error: Command failed"
        exit 1
    end
end

set release_dir "target/x86_64-unknown-linux-musl/release"
set bins bartos bartoc barto-cli

echo "Building MUSL binaries with Docker..."
run_step "docker run -v cargo-cache:/root/.cargo/registry -v (pwd):/home/rust/src -v ~/.gitconfig:/root/.gitconfig:ro -e SQLX_OFFLINE=true --rm -t blackdex/rust-musl:x86_64-musl-nightly cargo build --release --locked $features"

echo ""
echo "Fixing file ownership..."
run_step "sudo chown -R (whoami):(whoami) target/"

echo ""
echo "Copying binaries to home directory..."
for bin in $bins
    run_step "cp $release_dir/$bin ~/$bin"
end

echo ""
echo "✓ MUSL binaries built and copied to:"
for bin in $bins
    echo "  ~/$bin"
end
