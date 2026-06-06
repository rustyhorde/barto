#!/bin/sh
set -e

# DEB: $1=upgrade during upgrade, $1=remove during removal
# RPM: $1=1 during upgrade, $1=0 during removal
IS_UPGRADE=false
if [ "$1" = "upgrade" ] || [ "$1" = "1" ]; then IS_UPGRADE=true; fi

if systemctl is-active --quiet bartos 2>/dev/null; then
    systemctl stop bartos
fi
if systemctl is-active --quiet bartos-logrotate.timer 2>/dev/null; then
    systemctl stop bartos-logrotate.timer 2>/dev/null || true
fi

if ! $IS_UPGRADE; then
    if systemctl is-enabled --quiet bartos 2>/dev/null; then
        systemctl disable bartos
    fi
    if systemctl is-enabled --quiet bartos-logrotate.timer 2>/dev/null; then
        systemctl disable bartos-logrotate.timer 2>/dev/null || true
    fi
fi

systemctl daemon-reload >/dev/null 2>&1 || true
