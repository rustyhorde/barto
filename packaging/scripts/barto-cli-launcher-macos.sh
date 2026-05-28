#!/bin/sh
# Loads barto-cli secrets from the macOS Login Keychain and exports them as
# environment variables before exec-ing barto-cli.
#
# The Login Keychain is unlocked automatically at login.
#
# Store secrets once:
#   barto-cli secrets set BARTO_CLI_BARTOS_API_KEY
#
# See SECRETS.md for the full setup workflow.
set -e

load_secret() {
    security find-generic-password -s barto -a "$1" -w 2>/dev/null || true
}

if [ -z "${BARTO_CLI_BARTOS_API_KEY:-}" ]; then
    val=$(load_secret BARTO_CLI_BARTOS_API_KEY)
    if [ -n "$val" ]; then
        BARTO_CLI_BARTOS_API_KEY="$val"
        export BARTO_CLI_BARTOS_API_KEY
    fi
fi

exec barto-cli "$@"
