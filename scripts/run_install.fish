#!/usr/bin/env fish

function run_step
    echo "==> $argv"
    eval $argv
    if test $status -ne 0
        echo "Error: Command failed"
        exit 1
    end
end

run_step "cargo install --force --locked -p bartos"
run_step "cargo install --force --locked -p bartoc"
run_step "cargo install --force --locked -p barto-cli"

echo "✓ All binaries installed"
