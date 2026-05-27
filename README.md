# `barto` - A job scheduling system

[![codecov](https://codecov.io/gh/rustyhorde/barto/branch/master/graph/badge.svg?token=RAGSJQPZZ6)](https://codecov.io/gh/rustyhorde/barto)
[![CI](https://github.com/rustyhorde/barto/actions/workflows/barto.yml/badge.svg)](https://github.com/rustyhorde/barto/actions/workflows/barto.yml)
[![sponsor](https://img.shields.io/github/sponsors/crazysacx?logo=github-sponsors)](https://github.com/sponsors/CraZySacX)

## Overview

`barto` is a distributed, WebSocket-based job scheduling system composed of four components:

- **`bartos`** — Central scheduling server (Actix-web + MariaDB). Owns all schedule definitions and persists every job's output and exit status.
- **`bartoc`** — Remote worker client. Connects to `bartos` via WebSocket, receives schedule initializations, executes the configured commands, and streams results back.
- **`barto-cli`** — Command-line interface for querying and managing a running `bartos` instance.
- **`libbarto`** — Shared library providing the message protocol, `Realtime` scheduler, config types, TLS support, and tracing initialization.

**How it works**: `bartos` triggers scheduled commands at the configured times and sends them over WebSocket to the matching `bartoc` instance by name. `bartoc` executes each command, streams `stdout`/`stderr` output and exit status back to `bartos`, which persists everything to MariaDB for later querying via `barto-cli`.

All services are configured via TOML files located at `~/.config/<service>/<service>.toml` by default, with `BARTO*` environment variables available to override any TOML value.

## MSRV

1.95.0

## TLS & Certificate Pinning

`bartos` supports TLS for all WebSocket connections. `bartoc` and `barto-cli`
support **certificate pinning** — trusting only a specific CA certificate rather
than the full system/Mozilla root CA store. This prevents MITM attacks via a
compromised or malicious public CA.

### Generating a CA and server certificate

Two options are shown: `openssl` (ubiquitous) and `step` from
[Smallstep](https://smallstep.com/docs/step-cli/) (simpler API for PKI work).

#### Using `openssl`

```bash
# 1. Generate the CA key and self-signed CA certificate (valid 10 years)
openssl genrsa -out bartos-ca.key 4096
openssl req -new -x509 -days 3650 \
  -key bartos-ca.key \
  -out bartos-ca.pem \
  -subj "/CN=barto CA"

# 2. Generate the bartos server key and a certificate signing request (CSR).
#    Replace the CN and SAN values with your actual hostname/IP.
openssl genrsa -out bartos.key 4096
openssl req -new \
  -key bartos.key \
  -out bartos.csr \
  -subj "/CN=bartos.example.com"

# 3. Sign the server CSR with the CA (valid 1 year).
#    The subjectAltName extension is required by modern TLS clients (rustls).
#    Add all hostnames and IPs that clients will use to connect.
openssl x509 -req -days 365 \
  -in bartos.csr \
  -CA bartos-ca.pem \
  -CAkey bartos-ca.key \
  -CAcreateserial \
  -extfile <(printf "subjectAltName=DNS:bartos.example.com,IP:192.168.1.100") \
  -out bartos.pem
```

#### Using `step` (Smallstep CLI)

```bash
# 1. Generate the CA key and self-signed CA certificate
step certificate create "barto CA" bartos-ca.pem bartos-ca.key \
  --profile root-ca \
  --no-password --insecure

# 2. Generate the bartos server certificate signed by the CA.
#    Add all hostnames and IPs via --san flags.
step certificate create bartos.example.com bartos.pem bartos.key \
  --ca bartos-ca.pem --ca-key bartos-ca.key \
  --san bartos.example.com \
  --san 192.168.1.100 \
  --not-after 8760h \
  --no-password --insecure
```

### Configuring bartos (server)

Point `[actix.tls]` at the signed server certificate and key:

```toml
[actix.tls]
ip = "0.0.0.0"
port = "20000"
cert_file_path = "/etc/bartos/bartos.pem"
key_file_path  = "/etc/bartos/bartos.key"
```

### Configuring bartoc and barto-cli (clients)

Set `prefix = "wss"` and pin the CA certificate. Only connections whose server
certificate is signed by this CA will be accepted:

```toml
[bartos]
prefix  = "wss"
host    = "bartos.example.com"
port    = 20000
ca_cert = "/path/to/bartos-ca.pem"
```

Distribute `bartos-ca.pem` to every `bartoc` and `barto-cli` host. Keep
`bartos-ca.key` and `bartos.key` private and only on the `bartos` host.

> **Tip**: set restrictive permissions on key files:
> ```bash
> chmod 600 bartos-ca.key bartos.key
> ```

## `bartos` - The barto server

[![Crates.io](https://img.shields.io/crates/v/bartos.svg)](https://crates.io/crates/bartos)
[![Crates.io](https://img.shields.io/crates/l/bartos.svg)](https://crates.io/crates/bartos)
[![Crates.io](https://img.shields.io/crates/d/bartos.svg)](https://crates.io/crates/bartos)

### Configuration

`bartos` configuration is controlled via a toml file. By default this is located in the `bartos` directory rooted at the `dirs2` [config](https://docs.rs/dirs2/latest/dirs2/fn.config_dir.html) directory, i.e. `/home/<user>/.config/bartos/bartos.toml` on a Linux machine. The full path to the configuration file can also be specified as a command-line argument to `bartos`. See the
help output `bartos --help` for more details.

#### Format

```toml
# Actix Configuration
[actix]
# The number of actix worker to launch                      (REQUIRED)
workers = 8
# The ip address to listen on for the actix server          (REQUIRED)
ip = "0.0.0.0"
# The port to list on for the actix server                  (REQUIRED)
port = "20000"

# Actix TLS Configuration                                   (OPTIONAL)
[actix.tls]                        
# The ip address to listen on for a TLS connection          (REQUIRED)
ip = "0.0.0.1"                     
# The port to listen of for a TLS connection                (REQUIRED)
port = "20000"                     
# The full path to the Certificate PEM file                 (REQUIRED)
cert_file_path = "/path/cert.pem"
# The full path to the Private Key PEM file                 (REQUIRED)
key_file_path = "/path/key.pem"

# MariaDB Configuration                                     (REQUIRED)
[mariadb]
# The hostname of the database                              (REQUIRED)
host = "localhost"
# The port of the database, default 3306                    (OPTIONAL)
port = 3307
# The username for the database                             (REQUIRED)
username = "user"
# The password used to access the database                  (REQUIRED)
password = "pass"
# The database name                                         (REQUIRED)
database = "db"
# An & separated list of database directives                (OPTIONAL)
options = "ssl=true"

# stdout Tracing Configuration                              (REQUIRED)
[tracing.stdout]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"

# File Tracing Configuration                                (REQUIRED)
[tracing.file]
# The quiet level (more is less verbose output)             (REQUIRED)
quiet = 0
# The verbose level (more is verbose output)                (REQUIRED)
verbose = 3

# File Tracing Layer Configuration                          (REQUIRED)
[tracing.file.layer]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"

# An array of schedules for barto clients                   (REQUIRED)
# This is [schedules.<bartoc name>].
# This should match the name defined in your bartoc.toml.
[schedules.barto]
schedules = [
    { name = "echo", on_calendar = "*-*-* 10:R:R", cmds = [ "echo -n \"barto\"" ] }
]
```

The `on_calendar` format is outlined at [`Realtime`](https://docs.rs/libbarto/latest/libbarto/struct.Realtime.html)

### Command Line Usage
```text
A bartos server records information from bartoc instances and serves as a central hub for job scheduling

Usage: bartos [OPTIONS]

Options:
  -v, --verbose...
          Turn up logging verbosity (multiple will turn it up more)
  -q, --quiet...
          Turn down logging verbosity (multiple will turn it down more)
  -e, --enable-std-output
          Enable logging to stdout/stderr
  -c, --config-absolute-path <CONFIG_ABSOLUTE_PATH>
          Specify the absolute path to the config file
  -t, --tracing-absolute-path <TRACING_ABSOLUTE_PATH>
          Specify the absolute path to the tracing output file
  -h, --help
          Print help
  -V, --version
          Print version
```

## `bartoc` - The barto client

[![Crates.io](https://img.shields.io/crates/v/bartoc.svg)](https://crates.io/crates/bartoc)
[![Crates.io](https://img.shields.io/crates/l/bartoc.svg)](https://crates.io/crates/bartoc)
[![Crates.io](https://img.shields.io/crates/d/bartoc.svg)](https://crates.io/crates/bartoc)

### Configuration

`bartoc` configuration is controlled via a toml file. By default this is located in the `bartoc` directory rooted at the `dirs2` [config](https://docs.rs/dirs2/latest/dirs2/fn.config_dir.html) directory, i.e. `/home/<user>/.config/bartoc/bartoc.toml` on a Linux machine. The full path to the configuration file can also be specified as a command-line argument to `bartoc`. See the
help output `bartoc --help` for more details.

#### Format

```toml
# The name of the bartoc instance                           (REQUIRED)
name = "vader"
# The number of attempted re-connection attempts            (REQUIRED)
# after a disconnect
retry_count = "10"
# Optional connection timeout in seconds                    (OPTIONAL)
# client_timeout = 30
# How to handle missed scheduler ticks                      (OPTIONAL)
# Values: Burst (default), Delay, Skip
# missed_tick = "Burst"

# The bartos configuration                                  (REQUIRED)
[bartos]
# The websocket prefix, i.e. ws or wss.                     (REQUIRED)
# NOTE: wss requires TLS support on bartos
prefix = "wss"
# The hostname of the bartos instance                       (REQUIRED)
host = "localhost.ozias.net"
# The port of the bartos instance                           (REQUIRED)
port = 21526

# stdout Tracing Configuration                              (REQUIRED)
[tracing.stdout]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"

# File Tracing Configuration                                (REQUIRED)
[tracing.file]
# The quiet level (more is less verbose output)             (REQUIRED)
quiet = 0
# The verbose level (more is verbose output)                (REQUIRED)
verbose = 3

# File Tracing Layer Configuration                          (REQUIRED)
[tracing.file.layer]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"
```

### Command Line Usage
```text
A bartoc instance runs scheduled jobs and reports results back to a bartos instance

Usage: bartoc [OPTIONS]

Options:
  -v, --verbose...
          Turn up logging verbosity (multiple will turn it up more)
  -q, --quiet...
          Turn down logging verbosity (multiple will turn it down more)
  -e, --enable-std-output
          Enable logging to stdout/stderr
  -c, --config-absolute-path <CONFIG_ABSOLUTE_PATH>
          Specify the absolute path to the config file
  -t, --tracing-absolute-path <TRACING_ABSOLUTE_PATH>
          Specify the absolute path to the tracing output file
  -r, --redb-absolute-path <REDB_ABSOLUTE_PATH>
          Specify the absolute path to the redb database file
  -h, --help
          Print help
  -V, --version
          Print version
```

## `barto-cli` - The barto command line client

[![Crates.io](https://img.shields.io/crates/v/barto-cli.svg)](https://crates.io/crates/barto-cli)
[![Crates.io](https://img.shields.io/crates/l/barto-cli.svg)](https://crates.io/crates/barto-cli)
[![Crates.io](https://img.shields.io/crates/d/barto-cli.svg)](https://crates.io/crates/barto-cli)

### Configuration

`barto-cli` configuration is controlled via a toml file. By default this is located in the `barto-cli` directory rooted at the `dirs2` [config](https://docs.rs/dirs2/latest/dirs2/fn.config_dir.html) directory, i.e. `/home/<user>/.config/barto-cli/barto-cli.toml` on a Linux machine. The full path to the configuration file can also be specified as a command-line argument to `barto-cli`. See the
help output `barto-cli --help` for more details.

#### Format

```toml
# The name of the barto-cli instance                        (REQUIRED)
name = "vader-cli"

# The bartos instance configuration                         (REQUIRED)
[bartos]
# The websocket prefix, i.e. ws or wss.                     (REQUIRED)
# NOTE: wss requires TLS support on bartos
prefix = "wss"
# The hostname of the bartos instance                       (REQUIRED)
host = "localhost.ozias.net"
# The port of the bartos instance                           (REQUIRED)
port = 21526

# stdout Tracing Configuration                              (REQUIRED)
[tracing.stdout]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"

# File Tracing Configuration                                (REQUIRED)
[tracing.file]
# The quiet level (more is less verbose output)             (REQUIRED)
quiet = 0
# The verbose level (more is verbose output)                (REQUIRED)
verbose = 3

# File Tracing Layer Configuration                          (REQUIRED)
[tracing.file.layer]
# Should the target be included in tracing output           (REQUIRED)
with_target = true
# Should thread ids be included in the tracing output       (REQUIRED)
with_thread_ids = false
# Should thread names be included in the tracing output     (REQUIRED)
with_thread_names = false
# Should line numbers be included in the tracing output     (REQUIRED)
with_line_number = false
# Should the output level be included in the tracing output (REQUIRED)
with_level = true
# An comma separated list of tracing directives             (OPTIONAL)
directives = "actix_server=error,actix_tls=error"
```

### Command Line Usage
```text
A command line tool for requesting information from a bartos instance

Usage: barto-cli [OPTIONS] <COMMAND>

Commands:
  info     Display the bartos version information
  updates  Check for recent updates on a bartoc client
  cleanup  Perform cleanup of old database entries
  clients  List the currently connected clients
  query    Run a query on bartos
  list     List the output for the given command
  failed   List the jobs that failed
  cmd      Display output for the given command name across all clients
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...
          Turn up logging verbosity (multiple will turn it up more)
  -q, --quiet...
          Turn down logging verbosity (multiple will turn it down more)
  -e, --enable-std-output
          Enable logging to stdout/stderr
  -c, --config-absolute-path <CONFIG_ABSOLUTE_PATH>
          Specify the absolute path to the config file
  -t, --tracing-absolute-path <TRACING_ABSOLUTE_PATH>
          Specify the absolute path to the tracing output file
  -h, --help
          Print help
  -V, --version
          Print version
```
#### Info
```text
Display the bartos version information

Usage: barto-cli info [OPTIONS]

Options:
  -j, --json  Output the information in JSON format
  -h, --help  Print help
```

#### Updates
```text
Check for recent updates on a bartoc client

Usage: barto-cli updates --name <NAME> --update-kind <UPDATE_KIND>

Options:
  -n, --name <NAME>                The name of the bartoc client to check for recent updates
  -u, --update-kind <UPDATE_KIND>  Check for updates of the given kind
  -h, --help                       Print help
```

#### Cleanup
```text
Perform cleanup of old database entries

Usage: barto-cli cleanup

Options:
  -h, --help  Print help
```

#### Clients
```text
List the currently connected clients

Usage: barto-cli clients

Options:
  -h, --help  Print help
```

#### Query
```text
Run a query on bartos

Usage: barto-cli query --query <QUERY>

Options:
  -q, --query <QUERY>  The query to run on bartos
  -h, --help           Print help
```

#### List
```text
List the output for the given command

Usage: barto-cli list --name <NAME> [-c <CMD_NAME>]

Options:
  -n, --name <NAME>              The name of the bartoc client to check for recent updates
  -c, --cmd-name-opt <CMD_NAME>  The name of the command to list the output for
  -h, --help                     Print help
```

#### Failed
```text
List the jobs that failed

Usage: barto-cli failed

Options:
  -h, --help  Print help
```

#### Cmd
```text
Display output for the given command name across all clients

Usage: barto-cli cmd <CMD_NAME>

Arguments:
  <CMD_NAME>  The name of the command to display output for

Options:
  -h, --help  Print help
```

## `libbarto` - The shared library

[![docs.rs](https://docs.rs/libbarto/badge.svg)](https://docs.rs/libbarto)
[![Crates.io](https://img.shields.io/crates/v/libbarto.svg)](https://crates.io/crates/libbarto)
[![Crates.io](https://img.shields.io/crates/l/libbarto.svg)](https://crates.io/crates/libbarto)
[![Crates.io](https://img.shields.io/crates/d/libbarto.svg)](https://crates.io/crates/libbarto)
