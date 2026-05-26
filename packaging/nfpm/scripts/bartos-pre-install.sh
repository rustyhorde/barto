#!/bin/sh
set -e

if ! getent group bartos >/dev/null 2>&1; then
    groupadd --system bartos
fi

if ! getent passwd bartos >/dev/null 2>&1; then
    useradd --system \
        --gid bartos \
        --home-dir /var/lib/bartos \
        --no-create-home \
        --shell /sbin/nologin \
        --comment "bartos service user" \
        bartos
fi
