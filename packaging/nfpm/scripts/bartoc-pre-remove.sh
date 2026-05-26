#!/bin/sh
set -e

if systemctl is-active --quiet bartoc 2>/dev/null; then
    systemctl stop bartoc
fi

if systemctl is-enabled --quiet bartoc 2>/dev/null; then
    systemctl disable bartoc
fi

systemctl daemon-reload >/dev/null 2>&1 || true
