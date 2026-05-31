# Barto Secrets Management

Barto components use several shared secrets for authentication and message integrity.
This document explains how to store them securely without putting plaintext values in
configuration files.

## Secrets Overview

| Env Var | Component | Description |
|---|---|---|
| `BARTOS_HMAC_KEY` | bartos | Shared HMAC-SHA256 key for outgoing message authentication |
| `BARTOS_SIGNING_KEY` | bartos | Ed25519 private key for signing BartosToBartoc messages |
| `BARTOS_API_KEY` | bartos | Bearer token required on WebSocket upgrade |
| `BARTOS_MARIADB__PASSWORD` | bartos | MariaDB database password |
| `BARTOC_HMAC_KEY` | bartoc | Same shared HMAC-SHA256 key (must match `BARTOS_HMAC_KEY`) |
| `BARTOC_SERVER_PUBLIC_KEY` | bartoc | Ed25519 public key to verify messages from bartos |
| `BARTOC_BARTOS__API_KEY` | bartoc | Bearer token for WebSocket connection to bartos |
| `BARTO_CLI_BARTOS__API_KEY` | barto-cli | Bearer token for WebSocket connection to bartos |

All components already read these values from environment variables.  The config
system uses `<PREFIX>_<FIELD>` for flat (top-level) config fields and
`<PREFIX>_<STRUCT>__<FIELD>` (double underscore) for nested fields; no TOML changes
are required.

---

## bartos (Linux system service)

bartos runs as a system service before any user login, so it cannot access user
keychains.  Instead, use **systemd credentials** — secrets are encrypted at rest and
injected into the service process at start time.

### Encryption tiers (auto-selected by systemd)

| Tier | Protection at rest | Requires |
|---|---|---|
| Machine-key | Encrypted with `/var/lib/systemd/credential.secret` | systemd ≥ 250 |
| TPM2-bound | Additionally sealed to TPM2 PCRs | TPM2 device |

systemd picks the best available tier automatically.  Check TPM2 availability:

```sh
systemd-creds --has-tpm2   # exits 0 if TPM2 is usable
ls /dev/tpm0 /dev/tpmrm0 2>/dev/null
```

### Quick setup with `bartos-secrets-init`

```sh
bartos-secrets-init
```

The script prompts for each secret, encrypts it, and prints the
`SetCredentialEncrypted=` lines to add to the service.

### Manual setup

```sh
# Encrypt each secret (replace YOUR_VALUE with the actual secret):
printf 'YOUR_VALUE' | systemd-creds encrypt --name=hmac_key         -
printf 'YOUR_VALUE' | systemd-creds encrypt --name=signing_key      -
printf 'YOUR_VALUE' | systemd-creds encrypt --name=api_key          -
printf 'YOUR_VALUE' | systemd-creds encrypt --name=mariadb_password -
```

Create a drop-in file `/etc/systemd/system/bartos.service.d/secrets.conf`:

```ini
[Service]
SetCredentialEncrypted=hmac_key: \
        <paste blob from systemd-creds encrypt>
SetCredentialEncrypted=signing_key: \
        <paste blob>
SetCredentialEncrypted=api_key: \
        <paste blob>
SetCredentialEncrypted=mariadb_password: \
        <paste blob>
```

Then reload:

```sh
systemctl daemon-reload
systemctl restart bartos
```

The `bartos-launcher` wrapper (installed at `/usr/lib/bartos/bartos-launcher`) reads
`$CREDENTIALS_DIRECTORY/*` and exports each as a `BARTOS_*` environment variable
before exec-ing bartos.

---

## bartoc and barto-cli (user services / interactive tools)

### bartoc — choosing a secret storage method

bartoc runs as a systemd **user** service.  How secrets are stored depends on
whether bartoc starts before or after an interactive user login:

| Scenario | Recommended method |
|---|---|
| Lingering service (starts at boot, no login required) | `bartoc-secrets-init` → systemd user credentials (requires systemd ≥ 256) |
| Desktop only (user always logged in before service starts) | `barto-cli secrets set` → platform keychain |

Both methods are supported simultaneously.  **Systemd credentials take precedence
over the platform keychain** — `bartoc-launcher` exports credentials from
`$CREDENTIALS_DIRECTORY` first, then fills any remaining gaps from the platform
keychain.  Either source is ultimately exported as environment variables before
bartoc starts, which override any TOML configuration values.

#### Lingering services — systemd user credentials (systemd ≥ 256)

When `loginctl enable-linger` is set, bartoc starts at boot before any interactive
login.  The GNOME Keyring is not unlocked at that point, so `secret-tool` cannot
read secrets.  Use systemd user credentials instead.

> **Requires systemd ≥ 256.** Encrypted credentials in user services are not
> supported on older versions — the user manager cannot decrypt machine-key blobs
> at startup (`status=243/CREDENTIALS`).  For systemd 250–255, see the
> age-encrypted secrets section below, or use the platform keychain approach.

##### Interactive setup with `bartoc-secrets-init` (recommended)

```sh
bartoc-secrets-init
```

Add the output to a drop-in file:

```sh
mkdir -p ~/.config/systemd/user/bartoc.service.d
$EDITOR ~/.config/systemd/user/bartoc.service.d/secrets.conf
```

```ini
[Service]
SetCredentialEncrypted=hmac_key: \
        <blob from bartoc-secrets-init>
SetCredentialEncrypted=server_public_key: \
        <blob>
SetCredentialEncrypted=api_key: \
        <blob>
```

Then reload:

```sh
systemctl --user daemon-reload && systemctl --user restart bartoc
```

##### Manual setup (alternative to `bartoc-secrets-init`)

Use this if you prefer to encrypt secrets yourself rather than using the
interactive script.  The result is identical — a `secrets.conf` drop-in holding
`SetCredentialEncrypted=` blobs that `bartoc-launcher` reads at startup.

```sh
# Encrypt each secret (replace YOUR_VALUE with the actual secret):
printf 'YOUR_VALUE' | systemd-creds encrypt --user --name=hmac_key          - -
printf 'YOUR_VALUE' | systemd-creds encrypt --user --name=server_public_key - -
printf 'YOUR_VALUE' | systemd-creds encrypt --user --name=api_key           - -
```

Create `~/.config/systemd/user/bartoc.service.d/secrets.conf`:

```sh
mkdir -p ~/.config/systemd/user/bartoc.service.d
$EDITOR ~/.config/systemd/user/bartoc.service.d/secrets.conf
```

```ini
[Service]
SetCredentialEncrypted=hmac_key: \
        <paste blob from systemd-creds encrypt>
SetCredentialEncrypted=server_public_key: \
        <paste blob>
SetCredentialEncrypted=api_key: \
        <paste blob>
```

Then reload:

```sh
systemctl --user daemon-reload
systemctl --user restart bartoc
```

#### Lingering services — age-encrypted secrets (systemd 250–255)

For systems where upgrading to systemd ≥ 256 is not possible, `age` provides
encryption-at-rest without root or TPM2.  The two files produced by setup —
`~/.config/bartoc/age-identity` (the private key) and `~/.config/bartoc/secrets.age`
(the encrypted secrets bundle) — together serve the same role as `secrets.conf` does
in a systemd ≥ 256 setup: `bartoc-age.service` uses the identity file to decrypt the
secrets bundle at startup, then exports each value as an environment variable.

Install `age` first:

```sh
# Arch:
sudo pacman -S age
# Debian/Ubuntu:
sudo apt install age
# Fedora:
sudo dnf install age
```

Then run the interactive setup:

```sh
bartoc-age-secrets-init
```

This generates `~/.config/bartoc/age-identity` (0600) and
`~/.config/bartoc/secrets.age` (0600).  Enable the alternative service unit
(mutually exclusive with `bartoc.service`):

```sh
systemctl --user disable --now bartoc.service   # if currently active
systemctl --user enable  --now bartoc-age.service
systemctl --user daemon-reload
```

**Security**: `~/.config/bartoc/age-identity` (private key) and
`~/.config/bartoc/secrets.age` are both required to decrypt.  An attacker with
full read access to `~/.config/bartoc/` can decrypt — comparable to 0600
plaintext file security, but no plaintext secrets file exists on disk.  For
stronger at-rest protection, upgrade to systemd ≥ 256.

To update a secret later, re-run `bartoc-age-secrets-init` (existing values are
preserved for any prompt you leave blank).

#### Desktop sessions — platform keychain

`barto-cli secrets` is the cross-platform tool for managing client-side secrets.
It writes to and reads from the native keychain without requiring knowledge of
platform-specific CLI tools.

```sh
# Store a secret (prompts for value, no echo):
barto-cli secrets set BARTOC_HMAC_KEY
barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY
barto-cli secrets set BARTOC_BARTOS__API_KEY
barto-cli secrets set BARTO_CLI_BARTOS__API_KEY

# Check what is stored:
barto-cli secrets list

# Retrieve a value (prints to stdout):
barto-cli secrets get BARTOC_HMAC_KEY

# Delete a secret:
barto-cli secrets delete BARTOC_HMAC_KEY
```

Secrets are stored under service name `barto` in the platform keychain.

### Platform details

#### Linux — GNOME Keyring / KWallet

Requires `libsecret` and either GNOME Keyring or KWallet.

PAM auto-unlock (enabled by default on most desktop distributions):
- GNOME: `pam_gnome_keyring.so` in `/etc/pam.d/login`
- KDE:   `pam_kwallet5.so` in `/etc/pam.d/login`

The `bartoc-launcher` script at `/usr/lib/bartoc/bartoc-launcher` loads secrets in
priority order: systemd user credentials first (see above), then `secret-tool` for
any remaining gaps.  The bartoc systemd user service uses this launcher automatically.

The `barto-cli-launcher` script is installed at `/usr/bin/barto-cli` (the real binary
lives at `/usr/lib/barto-cli/barto-cli`).  Every `barto-cli` invocation transparently
loads `BARTO_CLI_BARTOS__API_KEY` from the keychain first.  If that secret is not
stored separately, the launcher falls back to `BARTOC_BARTOS__API_KEY` — useful when
bartos uses a single shared API key for both bartoc and barto-cli connections.

To verify your keychain is accessible:

```sh
secret-tool lookup service barto username BARTOC_HMAC_KEY
secret-tool lookup service barto username BARTO_CLI_BARTOS__API_KEY
```

#### macOS — Login Keychain

The Login Keychain is unlocked automatically at login.  Secrets are stored with
`security add-generic-password` under service name `barto`.

A reference launcher script is provided at
`packaging/scripts/bartoc-launcher-macos.sh`.  Adapt it for your launchd plist or
shell environment.

To verify:

```sh
security find-generic-password -s barto -a BARTOC_HMAC_KEY -w
```

#### Windows — PasswordVault (Credential Manager)

Secrets are stored in `Windows.Security.Credentials.PasswordVault`, which is tied
to the user account and accessible after login.

A reference launcher script is provided at `packaging/scripts/bartoc-launcher.ps1`.

To verify in PowerShell:

```powershell
$vault = New-Object Windows.Security.Credentials.PasswordVault
$cred  = $vault.Retrieve('barto', 'BARTOC_BARTOS__API_KEY')
$cred.RetrievePassword()
$cred.Password
```

---

## Generating secrets

```sh
# HMAC key (shared between bartos and bartoc — use the same value for both):
openssl rand -base64 32

# Ed25519 keypair (bartos holds the private key, bartoc holds the public key):
openssl genpkey -algorithm ed25519 -outform DER | tail -c 32 | base64   # private key
openssl pkey -pubout -outform DER -in <(openssl genpkey -algorithm ed25519) | tail -c 32 | base64  # public key

# API / Bearer token:
openssl rand -base64 32
```

---

## Removing secrets from TOML files

If you previously stored secrets in TOML files, remove them:

1. Edit `~/.config/bartos/bartos.toml` — delete or comment out `hmac_key`,
   `signing_key`, `api_key`, and `password` under `[mariadb]`.
2. Edit `~/.config/bartoc/bartoc.toml` — delete or comment out `hmac_key`,
   `server_public_key`, and `api_key` under `[bartos]`.
3. Edit `~/.config/barto-cli/barto-cli.toml` — delete or comment out `api_key`
   under `[bartos]`.

The env vars injected by the launcher scripts or keychain take precedence over
anything in the TOML files via the config system's priority order
(TOML < CLI args < env vars).
