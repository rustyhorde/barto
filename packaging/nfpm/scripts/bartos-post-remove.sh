#!/bin/sh
set -e

# Only remove user/group on full purge (DEB: $1="purge", RPM: $1=0)
if [ "$1" = "purge" ] || [ "$1" = "0" ]; then
    if getent passwd bartos >/dev/null 2>&1; then
        userdel bartos
    fi
    if getent group bartos >/dev/null 2>&1; then
        groupdel bartos
    fi
fi
