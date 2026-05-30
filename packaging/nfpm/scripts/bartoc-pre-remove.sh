#!/bin/sh
set -e

# DEB: $1=upgrade during upgrade, $1=remove during removal
# RPM: $1=1 during upgrade, $1=0 during removal
IS_UPGRADE=false
if [ "$1" = "upgrade" ] || [ "$1" = "1" ]; then IS_UPGRADE=true; fi

if $IS_UPGRADE; then
    if [ -d /var/lib/systemd/linger ]; then
        for _lf in /var/lib/systemd/linger/*; do
            [ -f "$_lf" ] || continue
            _user=$(basename "$_lf")
            for _svc in bartoc bartoc-age; do
                if systemctl --user -M "${_user}@.host" is-active --quiet "$_svc" 2>/dev/null; then
                    systemctl --user -M "${_user}@.host" stop "$_svc" 2>/dev/null || true
                fi
            done
        done
    fi
else
    echo ""
    echo "==> Removing bartoc..."
    echo "    If you have the service running, stop and disable it first:"
    echo "         systemctl --user disable --now bartoc"
    echo ""
fi
