# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

Barto is a 4-component websocket-based job scheduling system:

- **`bartos`** — Central server (Actix-web + MariaDB + websocket hub). Owns schedule definitions and stores all output/status.
- **`bartoc`** — Remote worker clients. Connect to `bartos` via websockets and execute scheduled commands.
- **`barto-cli`** — CLI tool for querying/managing a running `bartos` instance.
- **`libbarto`** — Shared library: message protocol, `Realtime` scheduler, config types, TLS, tracing.

**Data flow**: `bartos` triggers scheduled commands → sends to matching `bartoc` client → `bartoc` executes and streams output/status back → `bartos` persists to MariaDB (`output` and `exit_status` tables).

## Commands

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace --all-features

# Single test
cargo test --workspace --all-features <test_name>

# Lint (nightly required for full lint set)
cargo +nightly clippy --workspace --all-features

# Format
cargo fmt --all

# Coverage
cargo llvm-cov --lcov --output-path lcov.info

# Security audit
cargo audit
```

## Critical Patterns

### Message Protocol
All inter-component communication uses **bincode-serialized enums** over websockets. Protocol types live in `libbarto/src/message/`. Core types: `Data::Output`, `Data::Status`. Always use the `bon::Builder` pattern for construction.

### Realtime Scheduling (`libbarto/src/realtime/`)
Custom cron-like syntax inspired by systemd timers:
```toml
on_calendar = "*-*-* 10:R:R"        # every day at 10:XX:XX (R = random)
on_calendar = "Mon *-*-01 00:00:00" # first Monday of every month
```
Built-in shortcuts: `minutely`, `hourly`, `daily`, `weekly`, `monthly`, `quarterly`, `semiannually`, `yearly`.

### Configuration
- TOML files at `~/.config/{bartos,bartoc,barto-cli}/*.toml` (path overridable via CLI arg)
- `BARTO_*` env vars override TOML values
- Config structs use `bon::Builder` + `getset`
- `bartos` schedules are defined as `[schedules.<client_name>]` sections linking to `bartoc` instances by name

### Error Handling
- `anyhow::Result` for application-level errors, `thiserror::Error` for typed error enums
- CLI entry points use `libbarto::{clap_or_error, success}` for standardized exit codes

### Database
- MariaDB via SQLx with compile-time checked queries (`.sqlx/` holds query cache)
- Schema lives in `migrations/` — always add new SQL files there; never modify existing migrations
- `DATABASE_URL` in `.env` is used by SQLx at compile time (`mysql://barto:barto@localhost/barto`)

### Lints
The codebase uses an extensive nightly-gated lint configuration in every crate's `lib.rs`/`main.rs`. On nightly, virtually all warnings are treated as errors. The `rustversion` crate detects nightly in `build.rs`. The `unstable` feature flag enables additional nightly-only language features.

## Build System Notes

- **Workspace dependencies**: Add new deps to the workspace `Cargo.toml` first, then reference with `{ workspace = true }` in member crates.
- **MSRV**: Rust 1.91.1. CI tests against 1.91.1, stable, beta, and nightly.
- **`vergen-gix`**: Each crate's `build.rs` embeds git/build metadata at compile time (used for `barto-cli info` output).
- **`cargo audit`**: `.cargo/audit.toml` ignores RUSTSEC-2023-0071 (Marvin Attack in `rsa` via sqlx-mysql — no upstream fix).
