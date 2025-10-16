# `barto` - A job scheduling system

[![codecov](https://codecov.io/gh/rustyhorde/barto/branch/master/graph/badge.svg?token=RAGSJQPZZ6)](https://codecov.io/gh/rustyhorde/barto)
[![CI](https://github.com/rustyhorde/barto/actions/workflows/barto.yml/badge.svg)](https://github.com/rustyhorde/barto/actions/workflows/barto.yml)
[![sponsor](https://img.shields.io/github/sponsors/crazysacx?logo=github-sponsors)](https://github.com/sponsors/CraZySacX)

## MSRV
1.89.0

## `bartos` - The barto server
[![Crates.io](https://img.shields.io/crates/v/bartos.svg)](https://crates.io/crates/bartos)
[![Crates.io](https://img.shields.io/crates/l/bartos.svg)](https://crates.io/crates/bartos)
[![Crates.io](https://img.shields.io/crates/d/bartos.svg)](https://crates.io/crates/bartos)

### Configuration
`bartos` configuration is controlled via a toml file.  By default this is located in the `bartos` directory rooted at the `dirs2` [config](https://docs.rs/dirs2/latest/dirs2/fn.config_dir.html) directory, i.e. `/home/<user>/.config/bartos/bartos.toml` on a Linux machine.  The full path to the configuration file can also be specified as a command-line argument to `bartos`.  See the 
help output `bartos --help` for more details.

#### Format
```toml
# Actix Configuration
[actix]
workers = 8                        # Required
ip = "0.0.0.0"                     # Required
port = "20000"                     # Required

# Actix TLS Configuration
[actix.tls]                        # Optional
ip = "0.0.0.1"                     # Required, if TLS
port = "20000"                     # Required, if TLS
cert_file_path = "/path/cert.pem"  # Required, if TLS
key_file_path = "/path/key.pem"    # Required, if TLS

# MariaDB Configuration
[mariadb]
host = "localhost"                 # Required
port = 3307                        # Optional, default 3306
username = "user"                  # Required
password = "pass"                  # Required
database = "db"                    # Required
options = "ssl=true"               # Optional, & separated key-value pairs
output_table = "Output|OutputTest" # Required
status_table = "Status|StatusTest" # Required

# stdout Tracing Configuration
[tracing.stdout]
with_target = true                 # Required
with_thread_ids = false            # Required
with_thread_names = false          # Required
with_line_number = false           # Required
with_level = true                  # Required
directives = "actix_server=error"  # Required

# File Tracing Configuration
[tracing.file]
quiet = 0                          # Required
verbose = 3                        # Required

# File Tracing Layer Configuration
[tracing.file.layer]
with_target = true                 # Required
with_thread_ids = false            # Required
with_thread_names = false          # Required
with_line_number = false           # Required
with_level = true                  # Required
directives = "actix_server=error"  # Required

# Schedules for barto clients
[schedules]
```

## `bartoc` - The barto client
[![Crates.io](https://img.shields.io/crates/v/bartoc.svg)](https://crates.io/crates/bartoc)
[![Crates.io](https://img.shields.io/crates/l/bartoc.svg)](https://crates.io/crates/bartoc)
[![Crates.io](https://img.shields.io/crates/d/bartoc.svg)](https://crates.io/crates/bartoc)

## `barto-cli` - The barto command line client
[![Crates.io](https://img.shields.io/crates/v/barto-cli.svg)](https://crates.io/crates/barto-cli)
[![Crates.io](https://img.shields.io/crates/l/barto-cli.svg)](https://crates.io/crates/barto-cli)
[![Crates.io](https://img.shields.io/crates/d/barto-cli.svg)](https://crates.io/crates/barto-cli)

## `libbarto` - The shared library
[![docs.rs](https://docs.rs/libbarto/badge.svg)](https://docs.rs/libbarto)
[![Crates.io](https://img.shields.io/crates/v/libbarto.svg)](https://crates.io/crates/libbarto)
[![Crates.io](https://img.shields.io/crates/l/libbarto.svg)](https://crates.io/crates/libbarto)
[![Crates.io](https://img.shields.io/crates/d/libbarto.svg)](https://crates.io/crates/libbarto)

