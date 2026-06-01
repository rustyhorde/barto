#!/bin/sh
# Loads bartoc secrets from the macOS Login Keychain and exports them as
# environment variables before exec-ing bartoc.
#
# The Login Keychain is unlocked automatically at login.
# On first access for each secret, macOS may prompt for keychain access.
#
# Store secrets once:
#   barto-cli secrets set BARTOC_HMAC_KEY
#   barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY
#   barto-cli secrets set BARTOC_BARTOS__API_KEY
#
# See SECRETS.md for the full setup workflow.
set -e

if [ "$(id -u)" -eq 0 ]; then
    echo "bartoc-launcher-macos: must not run as root." >&2
    echo "Remove the system daemon and start as your user:" >&2
    echo "  sudo brew services stop bartoc" >&2
    echo "  brew services start bartoc" >&2
    exit 1
fi

load_secret() {
    security find-generic-password -s barto -a "$1" -w 2>/dev/null || true
}

if [ -z "${BARTOC_HMAC_KEY:-}" ]; then
    val=$(load_secret BARTOC_HMAC_KEY)
    if [ -n "$val" ]; then
        BARTOC_HMAC_KEY="$val"
        export BARTOC_HMAC_KEY
    fi
fi

if [ -z "${BARTOC_SERVER_PUBLIC_KEY:-}" ]; then
    val=$(load_secret BARTOC_SERVER_PUBLIC_KEY)
    if [ -n "$val" ]; then
        BARTOC_SERVER_PUBLIC_KEY="$val"
        export BARTOC_SERVER_PUBLIC_KEY
    fi
fi

if [ -z "${BARTOC_BARTOS__API_KEY:-}" ]; then
    val=$(load_secret BARTOC_BARTOS__API_KEY)
    if [ -n "$val" ]; then
        BARTOC_BARTOS__API_KEY="$val"
        export BARTOC_BARTOS__API_KEY
    fi
fi

exec "$(dirname "$0")/bartoc" "$@"
