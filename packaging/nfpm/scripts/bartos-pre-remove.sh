#!/bin/sh
set -e

if systemctl is-active --quiet bartos 2>/dev/null; then
    systemctl stop bartos
fi

if systemctl is-enabled --quiet bartos 2>/dev/null; then
    systemctl disable bartos
fi

systemctl daemon-reload >/dev/null 2>&1 || true
