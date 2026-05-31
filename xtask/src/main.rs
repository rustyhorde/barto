// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `cargo xtask dist <binary>`
//!
//! Generates shell completions (bash, zsh, fish) and a man page for the
//! given barto binary.  Each binary's output is written to `dist/<binary>/`.
//!
//! # Usage
//!
//! ```text
//! cargo xtask dist bartos
//! cargo xtask dist bartoc
//! cargo xtask dist barto-cli
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};
use clap::{Arg, ArgAction, Command};
use clap_complete::{Shell, generate_to};
use clap_mangen::Man;

fn main() -> Result<()> {
    let matches = Command::new("xtask")
        .subcommand_required(true)
        .subcommand(
            Command::new("dist")
                .about("Generate shell completions and man pages for a binary")
                .arg(
                    Arg::new("binary")
                        .required(true)
                        .help("Binary to generate artifacts for (bartos, bartoc, barto-cli)"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("dist", sub)) => {
            let binary = sub.get_one::<String>("binary").expect("required");
            dist(binary)
        }
        _ => bail!("unknown subcommand"),
    }
}

fn dist(binary: &str) -> Result<()> {
    let mut cmd = match binary {
        "bartos" => bartos_command(),
        "bartoc" => bartoc_command(),
        "barto-cli" => barto_cli_command(),
        other => bail!("unknown binary '{other}'; expected one of: bartos, bartoc, barto-cli"),
    };

    let out_dir = PathBuf::from("dist").join(binary);
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {}", out_dir.display()))?;

    generate_completions(binary, &mut cmd, &out_dir)?;
    generate_man_page(&cmd, &out_dir)?;
    copy_licenses(&out_dir)?;
    copy_example_config(binary, &out_dir)?;
    copy_systemd_unit(binary, &out_dir)?;
    copy_extras(binary, &out_dir)?;

    println!("Artifacts written to {}", out_dir.display());
    Ok(())
}

fn copy_licenses(out_dir: &Path) -> Result<()> {
    for name in ["LICENSE-MIT", "LICENSE-APACHE"] {
        fs::copy(name, out_dir.join(name))
            .with_context(|| format!("failed to copy {name} to {}", out_dir.display()))?;
    }
    Ok(())
}

fn copy_example_config(binary: &str, out_dir: &Path) -> Result<()> {
    let cfg = format!("{binary}.toml.example");
    let src = PathBuf::from(format!("packaging/arch/{binary}/examples/{cfg}"));
    if src.exists() {
        fs::copy(&src, out_dir.join(&cfg))
            .with_context(|| format!("failed to copy {}", src.display()))?;
    }
    Ok(())
}

fn copy_systemd_unit(binary: &str, out_dir: &Path) -> Result<()> {
    let unit = format!("{binary}.service");
    let src = PathBuf::from("dist").join(binary).join(&unit);
    // Service files live alongside xtask output; only copy if separately committed there.
    // For bartos and bartoc, the service file is a static asset committed under dist/.
    if src.exists() && src != out_dir.join(&unit) {
        fs::copy(&src, out_dir.join(&unit))
            .with_context(|| format!("failed to copy {}", src.display()))?;
    }
    Ok(())
}

fn copy_extras(binary: &str, out_dir: &Path) -> Result<()> {
    fs::copy("README.md", out_dir.join("README.md")).context("failed to copy README.md")?;

    if binary == "bartoc" {
        fs::copy(
            "packaging/nfpm/scripts/bartoc-secrets-init",
            out_dir.join("bartoc-secrets-init"),
        )
        .context("failed to copy bartoc-secrets-init")?;
        fs::copy(
            "packaging/nfpm/scripts/bartoc-age-secrets-init",
            out_dir.join("bartoc-age-secrets-init"),
        )
        .context("failed to copy bartoc-age-secrets-init")?;
        fs::copy(
            "packaging/scripts/bartoc-launcher.ps1",
            out_dir.join("bartoc-launcher.ps1"),
        )
        .context("failed to copy bartoc-launcher.ps1")?;
    }

    if binary == "bartos" {
        fs::copy(
            "packaging/nfpm/scripts/bartos-secrets-init",
            out_dir.join("bartos-secrets-init"),
        )
        .context("failed to copy bartos-secrets-init")?;

        fs::copy(
            "packaging/nfpm/scripts/barto-migrate",
            out_dir.join("barto-migrate"),
        )
        .context("failed to copy barto-migrate")?;

        let mig_out = out_dir.join("migrations");
        fs::create_dir_all(&mig_out).context("failed to create migrations dir")?;
        for entry in fs::read_dir("migrations").context("failed to read migrations/")? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("sql") {
                fs::copy(entry.path(), mig_out.join(entry.file_name()))
                    .with_context(|| format!("failed to copy {}", entry.path().display()))?;
            }
        }
    }
    Ok(())
}

// ── Completion generation ─────────────────────────────────────────────────────

fn generate_completions(binary: &str, cmd: &mut Command, out_dir: &Path) -> Result<()> {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        generate_to(shell, cmd, binary, out_dir).with_context(|| {
            format!(
                "failed to generate {} completions for {binary}",
                shell_name(shell)
            )
        })?;
    }
    Ok(())
}

fn shell_name(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
        _ => "unknown",
    }
}

// ── Man page generation ───────────────────────────────────────────────────────

fn generate_man_page(cmd: &Command, out_dir: &Path) -> Result<()> {
    let man = Man::new(cmd.clone());
    let file_name = format!("{}.1", cmd.get_name());
    let mut file = fs::File::create(out_dir.join(&file_name))
        .with_context(|| format!("failed to create man page file {file_name}"))?;
    man.render(&mut file)
        .with_context(|| format!("failed to render man page {file_name}"))?;
    Ok(())
}

// ── CLI command definitions ───────────────────────────────────────────────────
//
// These mirror the actual Cli structs in bartos/, bartoc/, and barto-cli/
// without importing those crates. Keep these in sync with any CLI changes.

/// `bartos` — central job scheduling server
fn bartos_command() -> Command {
    Command::new("bartos")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Central job scheduling server; receives job data from bartoc instances")
        .arg(verbose_arg())
        .arg(quiet_arg())
        .arg(enable_std_output_arg())
        .arg(config_absolute_path_arg())
        .arg(tracing_absolute_path_arg())
}

/// `bartoc` — scheduled job executor
fn bartoc_command() -> Command {
    Command::new("bartoc")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Scheduled job executor; reports results to bartos")
        .arg(verbose_arg())
        .arg(quiet_arg())
        .arg(enable_std_output_arg())
        .arg(config_absolute_path_arg())
        .arg(tracing_absolute_path_arg())
        .arg(
            Arg::new("redb-absolute-path")
                .short('r')
                .long("redb-absolute-path")
                .value_name("PATH")
                .help("Specify the absolute path to the redb database file"),
        )
}

/// `barto-cli` — CLI tool for querying bartos instances
fn barto_cli_command() -> Command {
    Command::new("barto-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Command-line tool for querying bartos instances")
        .arg(verbose_arg())
        .arg(quiet_arg())
        .arg(enable_std_output_arg())
        .arg(config_absolute_path_arg())
        .arg(tracing_absolute_path_arg())
        .subcommand_required(true)
        .subcommand(
            Command::new("secrets")
                .about(
                    "Manage barto secrets in the platform keychain (no bartos connection needed)",
                )
                .subcommand_required(true)
                .subcommand(
                    Command::new("set")
                        .about("Store a secret value in the platform keychain")
                        .arg(
                            Arg::new("key")
                                .value_name("KEY")
                                .required(true)
                                .help("Name of the secret (e.g. BARTOC_HMAC_KEY)"),
                        ),
                )
                .subcommand(
                    Command::new("get")
                        .about("Retrieve and print a secret from the platform keychain")
                        .arg(
                            Arg::new("key")
                                .value_name("KEY")
                                .required(true)
                                .help("Name of the secret to retrieve"),
                        ),
                )
                .subcommand(Command::new("list").about("List known barto secrets and their status"))
                .subcommand(
                    Command::new("delete")
                        .about("Delete a secret from the platform keychain")
                        .arg(
                            Arg::new("key")
                                .value_name("KEY")
                                .required(true)
                                .help("Name of the secret to delete"),
                        ),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("Display the bartos version information")
                .arg(
                    Arg::new("json")
                        .short('j')
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output the information in JSON format"),
                ),
        )
        .subcommand(
            Command::new("updates")
                .about("Check for recent updates on a bartoc client")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(true)
                        .help("The name of the bartoc client to check for recent updates"),
                )
                .arg(
                    Arg::new("update-kind")
                        .short('u')
                        .long("update-kind")
                        .value_name("KIND")
                        .required(true)
                        .help("The kind of updates to check for"),
                ),
        )
        .subcommand(Command::new("cleanup").about("Perform cleanup of old database entries"))
        .subcommand(Command::new("clients").about("List the currently connected clients"))
        .subcommand(
            Command::new("query").about("Run a query on bartos").arg(
                Arg::new("query")
                    .short('q')
                    .long("query")
                    .value_name("QUERY")
                    .required(true)
                    .help("The query to run on bartos"),
            ),
        )
        .subcommand(
            Command::new("list")
                .about("List the output for the given command")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(true)
                        .help("The name of the bartoc client to check for recent updates"),
                )
                .arg(
                    Arg::new("cmd-name-opt")
                        .short('c')
                        .long("cmd-name-opt")
                        .value_name("CMD")
                        .help("The name of the command to list the output for"),
                ),
        )
        .subcommand(Command::new("failed").about("List the jobs that failed"))
        .subcommand(
            Command::new("cmd")
                .about("Display output for the given command name across all clients")
                .arg(
                    Arg::new("cmd-name")
                        .value_name("CMD_NAME")
                        .required(true)
                        .help("The name of the command to display output for"),
                ),
        )
}

// ── Shared argument helpers ───────────────────────────────────────────────────

fn verbose_arg() -> Arg {
    Arg::new("verbose")
        .short('v')
        .long("verbose")
        .action(ArgAction::Count)
        .help("Turn up logging verbosity (multiple will turn it up more)")
        .conflicts_with("quiet")
}

fn quiet_arg() -> Arg {
    Arg::new("quiet")
        .short('q')
        .long("quiet")
        .action(ArgAction::Count)
        .help("Turn down logging verbosity (multiple will turn it down more)")
        .conflicts_with("verbose")
}

fn enable_std_output_arg() -> Arg {
    Arg::new("enable-std-output")
        .short('e')
        .long("enable-std-output")
        .action(ArgAction::SetTrue)
        .help("Enable logging to stdout/stderr")
}

fn config_absolute_path_arg() -> Arg {
    Arg::new("config-absolute-path")
        .short('c')
        .long("config-absolute-path")
        .value_name("PATH")
        .help("Specify the absolute path to the config file")
}

fn tracing_absolute_path_arg() -> Arg {
    Arg::new("tracing-absolute-path")
        .short('t')
        .long("tracing-absolute-path")
        .value_name("PATH")
        .help("Specify the absolute path to the tracing output file")
}
