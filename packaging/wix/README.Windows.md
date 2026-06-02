# Installing bartoc on Windows

## MSI installer (recommended)

Download `bartoc-VERSION-x86_64.msi` from the [latest release](https://github.com/rustyhorde/barto/releases/latest).

### Quick install (default service account)

```
msiexec /i bartoc-VERSION-x86_64.msi /l*v bartoc-install.log
```

This installs bartoc as a Windows service running under `NT AUTHORITY\LocalService`.

> **Note:** The `LocalService` account does not have access to user-specific credential
> storage. Configure secrets via environment variables on the service (see below) or
> run the service as a dedicated user account.

### Install with a dedicated service account (recommended for production)

1. Create a local Windows account for the service (**requires an Administrator terminal**):
   ```
   net user bartoc_svc YOUR_PASSWORD /add
   ```

2. Install the MSI specifying that account:
   ```
   msiexec /i bartoc-VERSION-x86_64.msi ^
     SERVICEACCOUNT=".\bartoc_svc" SERVICEPASSWORD="YOUR_PASSWORD" ^
     /l*v bartoc-install.log
   ```

   The MSI registers the service and sets directory permissions but does **not** start
   it — the service remains stopped until step 5 so that secrets and config can be
   configured first.

3. Log on as `bartoc_svc` and set the secrets:
   ```
   runas /user:bartoc_svc "barto-cli secrets set BARTOC_HMAC_KEY"
   runas /user:bartoc_svc "barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY"
   runas /user:bartoc_svc "barto-cli secrets set BARTOC_BARTOS__API_KEY"
   ```

4. Copy and edit the example config:
   ```
   copy "%ProgramData%\bartoc\bartoc.toml.example" "%ProgramData%\bartoc\bartoc.toml"
   notepad "%ProgramData%\bartoc\bartoc.toml"
   ```

   **File paths used by the service** (baked into the service arguments at install time):

   | Purpose | Path |
   |---------|------|
   | Configuration | `%ProgramData%\bartoc\bartoc.toml` |
   | Log file | `%ProgramData%\bartoc\bartoc.log` (created on first start) |
   | Local database | `%ProgramData%\bartoc\bartoc.redb` (created on first start) |

5. Start the service:
   ```
   sc start bartoc
   ```

## Console mode (no installer)

Download `bartoc-x86_64-pc-windows-msvc.exe` from the release assets.

Run directly (without `--service`) to operate as a normal console application:

```
bartoc.exe --config-absolute-path C:\path\to\bartoc.toml
```

Secrets are loaded from Windows Credential Manager automatically at startup when not
already set as environment variables. Store them once with `barto-cli`:

```
barto-cli secrets set BARTOC_HMAC_KEY
barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY
barto-cli secrets set BARTOC_BARTOS__API_KEY
```

Alternatively, use `bartoc-launcher.ps1` (included in `%ProgramData%\bartoc\`) which
loads secrets from the Windows Credential Manager (PasswordVault) before launching
bartoc — the same mechanism used by `barto-cli secrets set`.

## Manual service install (without MSI)

```
sc create bartoc ^
  binPath= "\"C:\Program Files\barto\bartoc\bartoc.exe\" --service --config-absolute-path \"C:\ProgramData\bartoc\bartoc.toml\" --tracing-absolute-path \"C:\ProgramData\bartoc\bartoc.log\" --redb-absolute-path \"C:\ProgramData\bartoc\bartoc.redb\"" ^
  obj= ".\bartoc_svc" ^
  password= "YOUR_PASSWORD" ^
  start= delayed-auto ^
  DisplayName= "Barto Worker Client"

sc description bartoc "Executes scheduled jobs and reports results to a bartos instance."
sc start bartoc
```

## Service management

```
sc query bartoc       # check status
sc stop bartoc        # graceful stop
sc start bartoc       # start
sc delete bartoc      # remove (stop first)
```

## Upgrading

The MSI installer handles in-place upgrades automatically. Run the new MSI the same
way as the original install:

```
msiexec /i bartoc-NEW-VERSION-x86_64.msi ^
  SERVICEACCOUNT=".\bartoc_svc" SERVICEPASSWORD="YOUR_PASSWORD" ^
  /l*v bartoc-upgrade.log
```

The installer stops the running service before replacing files and leaves it stopped
after the upgrade completes. Restart it once the upgrade finishes:

```
sc start bartoc
```

> **Note:** Secrets (Windows Credential Manager) and config (`%ProgramData%\bartoc\bartoc.toml`)
> are preserved across upgrades — no re-configuration is needed.

## Uninstall (MSI)

```
msiexec /x bartoc-VERSION-x86_64.msi
```

Or via **Settings → Apps → Installed apps → bartoc → Uninstall**.
