# Barto - Websocket Job Scheduling System

## Architecture Overview

Barto is a **4-component system** for distributed job scheduling via websockets:

- **`bartos`** - Central server with Actix-web + MariaDB storage + websocket hub
- **`bartoc`** - Remote worker clients that connect via websockets and execute scheduled commands  
- **`barto-cli`** - Command-line interface for querying/managing the system
- **`libbarto`** - Shared library containing message protocols, realtime scheduling, and config

### Key Data Flow
1. `bartos` server maintains schedules defined in TOML config
2. `bartoc` clients connect via websockets and register their capabilities
3. `bartos` sends commands to appropriate `bartoc` instances based on schedule triggers
4. `bartoc` executes commands and streams output/status back via websockets
5. `bartos` stores all output/status in MariaDB (`output` and `exit_status` tables)

## Development Workflow

### Build & Test Commands
```bash
# Build all workspace members (preferred over individual crate builds)
cargo build --workspace

# Run tests with full feature matrix (like CI)
cargo test --workspace --all-features

# Check with nightly-specific lints (project uses extensive lint configuration)
cargo +nightly clippy --workspace --all-features

# Generate coverage (project has codecov integration)
cargo llvm-cov --lcov --output-path lcov.info
```

### Database Development
- **Always use migrations**: Add new `.sql` files to `migrations/` following the naming pattern
- **Use SQLx**: All database queries use compile-time checked SQLx macros
- Tables: `output` (command stdout/stderr), `exit_status` (command success/failure)

## Critical Patterns

### Message Protocol
All inter-component communication uses **bincode-serialized enums** over websockets:
- See `libbarto/src/message/` for protocol definitions
- `Data::Output` and `Data::Status` are core message types
- Always use the builder pattern: `Output::builder().bartoc_uuid(...).build()`

### Realtime Scheduling
**Custom cron-like syntax** in `libbarto/src/realtime/`:
```toml
# systemd-timer inspired format: [day_of_week] year-month-day hour:minute:second  
on_calendar = "*-*-* 10:R:R"  # Every day at 10:XX:XX (random minute/second)
on_calendar = "Mon *-*-01 00:00:00"  # First Monday of every month
```
- `R` = random value within valid range
- Built-in shortcuts: `minutely`, `hourly`, `daily`, `weekly`, `monthly`, `quarterly`, `yearly`

### Configuration System
- **TOML-first**: All components read from `~/.config/{bartos,bartoc,barto-cli}/*.toml`
- **Environment override**: Use `BARTO_*` prefixed env vars
- **Validation**: Config structs use `bon::Builder` + `getset` for type safety
- Server schedules defined as: `[schedules.<client_name>] = [{ name = "task", on_calendar = "...", cmds = [...] }]`

### Error Handling
- **anyhow + thiserror**: `anyhow::Result` for application errors, `thiserror::Error` for typed errors
- **clap integration**: Use `libbarto::{clap_or_error, success}` for CLI exit codes
- **Websocket errors**: Always handle connection drops gracefully

### Testing Patterns
- **proptest**: Extensive property-based testing for realtime scheduling (see `proptest-regressions/`)
- **Bincode roundtrip**: Test all message types for serialization stability
- **Integration**: Use temporary databases and mock websockets for component testing

## Build System Notes

- **Nightly-dependent**: Uses conditional compilation for nightly-only lints via `build.rs`
- **Git metadata**: `vergen-gix` embeds commit info at build time  
- **Workspace dependencies**: Always add deps to workspace `Cargo.toml` first, then reference
- **MSRV**: Rust 1.89.0 minimum, but CI tests against stable/beta/nightly

## Key Files

- `libbarto/src/lib.rs` - Master export list for shared types
- `libbarto/src/realtime/mod.rs` - Core scheduling engine
- `libbarto/src/message/shared/output.rs` - Primary data structures
- `migrations/*.sql` - Database schema evolution
- `.github/workflows/barto.yml` - CI pipeline with extensive cross-platform testing