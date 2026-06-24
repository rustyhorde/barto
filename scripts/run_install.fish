#!/usr/bin/env fish

function run_step
    echo "==> $argv"
    eval $argv
    if test $status -ne 0
        echo "Error: Command failed"
        exit 1
    end
end

run_step "cargo install --path bartos --force --locked"
run_step "cargo install --path bartoc --force --locked"
run_step "cargo install --path barto-cli --force --locked"

echo "✓ All binaries installed"
