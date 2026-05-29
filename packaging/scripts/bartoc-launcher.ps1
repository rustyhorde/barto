# bartoc-launcher.ps1
# Loads bartoc secrets from the Windows Credential Manager (PasswordVault) and
# sets them as environment variables before launching bartoc.
#
# The PasswordVault is tied to the Windows user account and accessible without
# a separate password prompt after login.
#
# Store secrets once:
#   barto-cli secrets set BARTOC_HMAC_KEY
#   barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY
#   barto-cli secrets set BARTOC_BARTOS__API_KEY
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

if (-not $env:BARTOC_HMAC_KEY) {
    $val = Get-BartoSecret 'BARTOC_HMAC_KEY'
    if ($val) { $env:BARTOC_HMAC_KEY = $val }
}

if (-not $env:BARTOC_SERVER_PUBLIC_KEY) {
    $val = Get-BartoSecret 'BARTOC_SERVER_PUBLIC_KEY'
    if ($val) { $env:BARTOC_SERVER_PUBLIC_KEY = $val }
}

if (-not $env:BARTOC_BARTOS__API_KEY) {
    $val = Get-BartoSecret 'BARTOC_BARTOS__API_KEY'
    if ($val) { $env:BARTOC_BARTOS__API_KEY = $val }
}

& bartoc @args
