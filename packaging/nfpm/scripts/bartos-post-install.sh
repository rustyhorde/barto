#!/bin/sh
set -e

systemctl daemon-reload >/dev/null 2>&1 || true

echo ""
echo "==> bartos installed successfully!"
echo ""
echo "    Before starting the service, configure the database connection:"
echo "    1. Copy and edit the example config:"
echo "         cp /usr/share/doc/bartos/examples/bartos.toml.example /etc/bartos/bartos.toml"
echo "         \$EDITOR /etc/bartos/bartos.toml"
echo "    2. Run database migrations:"
echo "         DATABASE_URL='mariadb://user:pass@localhost/barto' barto-migrate"
echo "    3. Enable and start the service:"
echo "         systemctl enable --now bartos"
echo "    Optional: TLS & Security"
echo "         To enable TLS, set [actix.tls] in bartos.toml with cert_file_path and key_file_path."
echo "         To require mutual TLS (client certs), also set client_ca_cert."
echo "         See: https://github.com/rustyhorde/barto#tls--certificate-pinning"
echo ""
