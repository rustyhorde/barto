#!/bin/sh
# Loads barto-cli secrets from the macOS Login Keychain and exports them as
# environment variables before exec-ing barto-cli.
#
# The Login Keychain is unlocked automatically at login.
#
# Store secrets once:
#   barto-cli secrets set BARTO_CLI_BARTOS__API_KEY
#
# If barto-cli is co-located with bartoc and bartos uses a single shared API key,
# you may store it only once under BARTOC_BARTOS__API_KEY; this launcher will fall
# back to that value automatically.
#
# See SECRETS.md for the full setup workflow.
#
# NOTE: When installing this launcher, place the real barto-cli binary at a
# different path (e.g., /usr/local/lib/barto-cli/barto-cli) and update the exec
# line below to avoid infinite recursion.
set -e

load_secret() {
    security find-generic-password -s barto -a "$1" -w 2>/dev/null || true
}

if [ -z "${BARTO_CLI_BARTOS__API_KEY:-}" ]; then
    val=$(load_secret BARTO_CLI_BARTOS__API_KEY)
    if [ -n "$val" ]; then
        BARTO_CLI_BARTOS__API_KEY="$val"
        export BARTO_CLI_BARTOS__API_KEY
    fi
fi

# Fall back to bartoc's key when co-located (same value stored once)
if [ -z "${BARTO_CLI_BARTOS__API_KEY:-}" ] && [ -n "${BARTOC_BARTOS__API_KEY:-}" ]; then
    BARTO_CLI_BARTOS__API_KEY="$BARTOC_BARTOS__API_KEY"
    export BARTO_CLI_BARTOS__API_KEY
fi

exec /usr/local/lib/barto-cli/barto-cli "$@"
