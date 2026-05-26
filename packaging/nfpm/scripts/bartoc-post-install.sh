#!/bin/sh
set -e

systemctl daemon-reload >/dev/null 2>&1 || true

echo ""
echo "==> bartoc installed successfully!"
echo ""
echo "    Before starting the service, configure the bartos connection:"
echo "    1. Edit /usr/share/doc/bartoc/examples/bartoc.toml.example"
echo "       and copy it to /etc/bartoc/bartoc.toml (or ~/.config/bartoc/bartoc.toml)"
echo "    2. Enable and start the service:"
echo "         systemctl enable --now bartoc"
echo ""
