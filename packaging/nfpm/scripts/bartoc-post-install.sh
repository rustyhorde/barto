#!/bin/sh
set -e

systemctl daemon-reload >/dev/null 2>&1 || true

echo ""
echo "==> bartoc installed successfully!"
echo ""
echo "    Before starting the service, configure the bartos connection:"
echo "    1. Copy and edit the example config:"
echo "         cp /usr/share/doc/bartoc/examples/bartoc.toml.example /etc/bartoc/bartoc.toml"
echo "         \$EDITOR /etc/bartoc/bartoc.toml"
echo "    2. Enable and start the service:"
echo "         systemctl enable --now bartoc"
echo ""
