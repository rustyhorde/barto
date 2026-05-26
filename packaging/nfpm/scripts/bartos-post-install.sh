#!/bin/sh
set -e

systemctl daemon-reload >/dev/null 2>&1 || true

echo ""
echo "==> bartos installed successfully!"
echo ""
echo "    Before starting the service, configure the database connection:"
echo "    1. Edit /usr/share/doc/bartos/examples/bartos.toml.example"
echo "       and copy it to /etc/bartos/bartos.toml (or ~/.config/bartos/bartos.toml)"
echo "    2. Run database migrations:"
echo "         DATABASE_URL='mariadb://user:pass@localhost/barto' barto-migrate"
echo "    3. Enable and start the service:"
echo "         systemctl enable --now bartos"
echo ""
