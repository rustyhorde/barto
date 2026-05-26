#!/bin/sh
set -e

if ! getent group bartoc >/dev/null 2>&1; then
    groupadd --system bartoc
fi

if ! getent passwd bartoc >/dev/null 2>&1; then
    useradd --system \
        --gid bartoc \
        --home-dir /var/lib/bartoc \
        --no-create-home \
        --shell /sbin/nologin \
        --comment "bartoc service user" \
        bartoc
fi
