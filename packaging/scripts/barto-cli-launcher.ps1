# barto-cli-launcher.ps1
# Loads barto-cli secrets from the Windows Credential Manager (PasswordVault) and
# sets them as environment variables before launching barto-cli.
#
# The PasswordVault is tied to the Windows user account and accessible without
# a separate password prompt after login.
#
# Store secrets once:
#   barto-cli secrets set BARTO_CLI_BARTOS__API_KEY
#
# If barto-cli is co-located with bartoc and bartos uses a single shared API key,
# you may store it only once under BARTOC_BARTOS__API_KEY; this launcher will fall
# back to that value automatically.
#
# See SECRETS.md for the full setup workflow.

Add-Type -AssemblyName Windows.Security

function Get-BartoSecret {
    param([string]$Key)
    try {
        $vault = New-Object Windows.Security.Credentials.PasswordVault
        $cred = $vault.Retrieve('barto', $Key)
        $cred.RetrievePassword()
        return $cred.Password
    } catch {
        return $null
    }
}

if (-not $env:BARTO_CLI_BARTOS__API_KEY) {
    $val = Get-BartoSecret 'BARTO_CLI_BARTOS__API_KEY'
    if ($val) { $env:BARTO_CLI_BARTOS__API_KEY = $val }
}

if (-not $env:BARTO_CLI_BARTOS__API_KEY -and $env:BARTOC_BARTOS__API_KEY) {
    $env:BARTO_CLI_BARTOS__API_KEY = $env:BARTOC_BARTOS__API_KEY
}

& barto-cli @args
