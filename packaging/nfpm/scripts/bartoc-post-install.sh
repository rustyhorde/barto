#!/bin/sh
set -e

# DEB: $1=configure; $2=old_version on upgrade, empty on fresh install
# RPM: $1=1 on fresh install, $1=2 on upgrade
IS_UPGRADE=false
if [ "$1" = "configure" ] && [ -n "${2:-}" ]; then IS_UPGRADE=true; fi
if [ "$1" = "2" ]; then IS_UPGRADE=true; fi

if $IS_UPGRADE; then
    if [ -d /var/lib/systemd/linger ]; then
        for _lf in /var/lib/systemd/linger/*; do
            [ -f "$_lf" ] || continue
            _user=$(basename "$_lf")
            systemctl --user -M "${_user}@.host" daemon-reload 2>/dev/null || true
            if systemctl --user -M "${_user}@.host" is-enabled --quiet bartoc 2>/dev/null; then
                echo "==> Restarting bartoc service for ${_user}..."
                if systemctl --user -M "${_user}@.host" restart bartoc; then
                    echo "==> bartoc restarted successfully."
                else
                    echo "==> Warning: failed to restart bartoc for ${_user}. Restart manually: systemctl --user restart bartoc"
                fi
            elif systemctl --user -M "${_user}@.host" is-enabled --quiet bartoc-age 2>/dev/null; then
                echo "==> Restarting bartoc-age service for ${_user}..."
                if systemctl --user -M "${_user}@.host" restart bartoc-age; then
                    echo "==> bartoc-age restarted successfully."
                else
                    echo "==> Warning: failed to restart bartoc-age for ${_user}. Restart manually: systemctl --user restart bartoc-age"
                fi
            else
                echo ""
                echo "==> bartoc upgraded."
                echo "    Run 'systemctl --user daemon-reload && systemctl --user enable --now bartoc' when ready."
                echo ""
            fi
            if systemctl --user -M "${_user}@.host" is-enabled --quiet bartoc-logrotate.timer 2>/dev/null; then
                echo "==> Restarting bartoc-logrotate.timer for ${_user}..."
                if systemctl --user -M "${_user}@.host" restart bartoc-logrotate.timer 2>/dev/null; then
                    echo "==> bartoc-logrotate.timer restarted successfully."
                else
                    echo "==> Warning: failed to restart bartoc-logrotate.timer for ${_user}."
                fi
            fi
        done
    fi
else
    echo ""
    echo "==> bartoc installed successfully!"
    echo ""
    echo "    Before starting the service, configure the bartos connection:"
    echo "    1. Copy and edit the example config:"
    echo "         mkdir -p ~/.config/bartoc"
    echo "         cp /usr/share/doc/bartoc/examples/bartoc.toml.example ~/.config/bartoc/bartoc.toml"
    echo "         \$EDITOR ~/.config/bartoc/bartoc.toml"
    echo "    2. Store secrets — choose one method:"
    echo ""
    echo "       A) Systemd user credentials (recommended for lingering services started at boot):"
    echo "            bartoc-secrets-init"
    echo "         Adds SetCredentialEncrypted= lines to:"
    echo "            ~/.config/systemd/user/bartoc.service.d/secrets.conf"
    echo ""
    echo "       B) Platform keychain (suitable when user is always logged in at service start):"
    echo "            barto-cli secrets set BARTOC_HMAC_KEY"
    echo "            barto-cli secrets set BARTOC_SERVER_PUBLIC_KEY   # if using Ed25519 signing"
    echo "            barto-cli secrets set BARTOC_BARTOS__API_KEY      # if using Bearer token auth"
    echo ""
    echo "    3. Enable lingering so bartoc starts at boot (optional but recommended):"
    echo "         loginctl enable-linger \$USER"
    echo "    4. Enable and start the service:"
    echo "         systemctl --user daemon-reload"
    echo "         systemctl --user enable --now bartoc"
    echo "    5. Enable daily log rotation:"
    echo "         systemctl --user enable --now bartoc-logrotate.timer"
    echo "    Optional: TLS & Certificate Pinning"
    echo "         Certificate pinning:  set ca_cert in [bartos] to pin your bartos CA cert."
    echo "         Mutual TLS:           set client_cert and client_key in [bartos]."
    echo "         See SECRETS.md and: https://github.com/rustyhorde/barto#secrets-management"
    echo ""
fi
