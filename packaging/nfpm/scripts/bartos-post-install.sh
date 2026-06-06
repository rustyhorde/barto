#!/bin/sh
set -e

# DEB: $1=configure; $2=old_version on upgrade, empty on fresh install
# RPM: $1=1 on fresh install, $1=2 on upgrade
IS_UPGRADE=false
if [ "$1" = "configure" ] && [ -n "${2:-}" ]; then IS_UPGRADE=true; fi
if [ "$1" = "2" ]; then IS_UPGRADE=true; fi

systemctl daemon-reload >/dev/null 2>&1 || true

if $IS_UPGRADE; then
    if systemctl is-enabled --quiet bartos 2>/dev/null; then
        echo "==> Restarting bartos service..."
        systemctl restart bartos
        echo "==> bartos restarted successfully."
    fi
    if systemctl is-enabled --quiet bartos-logrotate.timer 2>/dev/null; then
        echo "==> Restarting bartos-logrotate.timer..."
        systemctl restart bartos-logrotate.timer 2>/dev/null || true
    fi
else
    echo ""
    echo "==> bartos installed successfully!"
    echo ""
    echo "    Before starting the service, configure the database connection:"
    echo "    1. Copy and edit the example config:"
    echo "         cp /usr/share/doc/bartos/examples/bartos.toml.example /etc/bartos/bartos.toml"
    echo "         \$EDITOR /etc/bartos/bartos.toml"
    echo "    2. Run database migrations:"
    echo "         DATABASE_URL='mariadb://user:pass@localhost/barto' barto-migrate"
    echo "    3. Set up secrets (HMAC key, signing key, API key, DB password):"
    echo "         bartos-secrets-init"
    echo "       Follow the prompts and add the printed SetCredentialEncrypted= lines to:"
    echo "         /etc/systemd/system/bartos.service.d/secrets.conf"
    echo "    4. Enable and start the service:"
    echo "         systemctl daemon-reload"
    echo "         systemctl enable --now bartos"
    echo "    5. Enable daily log rotation:"
    echo "         systemctl enable --now bartos-logrotate.timer"
    echo "    Optional: TLS & Security"
    echo "         To enable TLS, set [actix.tls] in bartos.toml with cert_file_path and key_file_path."
    echo "         To require mutual TLS (client certs), also set client_ca_cert."
    echo "         See SECRETS.md and: https://github.com/rustyhorde/barto#secrets-management"
    echo ""
fi
