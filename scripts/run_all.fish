#!/usr/bin/env fish

set run_tests true
set run_coverage true
set run_docs true
set run_install true
set run_musl true
set musl_features ""
set run_clean false
set help false

for arg in $argv
    if test "$arg" = "--no-test"
        set run_tests false
    else if test "$arg" = "--no-coverage"
        set run_coverage false
    else if test "$arg" = "--no-docs"
        set run_docs false
    else if test "$arg" = "--no-install"
        set run_install false
    else if test "$arg" = "--no-musl"
        set run_musl false
    else if test "$arg" = "--unstable"
        set musl_features "--unstable"
    else if test "$arg" = "--clean"
        set run_clean true
    else if test "$arg" = "--help"
        set help true
    end
end

if test $help = true
    echo "Usage: run_all.fish [OPTIONS]"
    echo ""
    echo "Run the full local CI pipeline."
    echo ""
    echo "Options:"
    echo "  --no-test       Skip tests and coverage"
    echo "  --no-coverage   Skip coverage reports (keep tests)"
    echo "  --no-docs       Skip cargo doc"
    echo "  --no-install    Skip run_install.fish"
    echo "  --no-musl       Skip run_musl.fish"
    echo "  --unstable      Build MUSL with --features unstable"
    echo "  --clean         Run cargo clean at end"
    echo "  --help          Show this help message"
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

set script_dir (dirname (status filename))

echo ""
echo "=== Step 1: Format code ==="
run_step "cargo fmt --all"

echo ""
echo "=== Step 2: Check formatting ==="
run_step "cargo fmt --all -- --check"

echo ""
echo "=== Step 3: Clippy lint ==="
run_step "cargo matrix clippy --all-targets -- -Dwarnings"

echo ""
echo "=== Step 4: Build ==="
run_step "cargo matrix build"

if test $run_tests = true
    echo ""
    echo "=== Step 5: Tests ==="
    run_step "cargo nextest run -p libbarto -p bartoc -p barto-cli"

    echo ""
    echo "=== Step 6: Documentation ==="
    if test $run_docs = true
        run_step "cargo doc -p libbarto"
    end

    if test $run_coverage = true
        echo ""
        echo "=== Step 7: Coverage ==="
        run_step "cargo llvm-cov nextest -F unstable --no-report --exclude bartos --exclude xtask --workspace"

        echo ""
        echo "=== Step 8: Coverage report (LCOV) ==="
        run_step "cargo llvm-cov report --lcov --output-path lcov.info"

        echo ""
        echo "=== Step 9: Coverage report (HTML) ==="
        run_step "cargo llvm-cov report --html"
    end
else
    if test $run_docs = true
        echo ""
        echo "=== Step 6: Documentation ==="
        run_step "cargo doc -p libbarto"
    end
end

if test $run_install = true
    echo ""
    echo "=== Step 10: Install ==="
    run_step "$script_dir/run_install.fish"
end

if test $run_musl = true
    echo ""
    echo "=== Step 11: Build MUSL ==="
    run_step "$script_dir/run_musl.fish $musl_features"
end

if test $run_clean = true
    echo ""
    echo "=== Step 12: Clean ==="
    run_step "cargo clean"
end

echo ""
echo "✓ CI pipeline complete"
