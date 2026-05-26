#!/bin/sh
set -e

echo ""
echo "==> bartoc installed successfully!"
echo ""
echo "    Before starting the service, configure the bartos connection:"
echo "    1. Copy and edit the example config:"
echo "         mkdir -p ~/.config/bartoc"
echo "         cp /usr/share/doc/bartoc/examples/bartoc.toml.example ~/.config/bartoc/bartoc.toml"
echo "         \$EDITOR ~/.config/bartoc/bartoc.toml"
echo "    2. Enable and start the service:"
echo "         systemctl --user daemon-reload"
echo "         systemctl --user enable --now bartoc"
echo "    3. Enable daily log rotation:"
echo "         systemctl --user enable --now bartoc-logrotate.timer"
echo ""
