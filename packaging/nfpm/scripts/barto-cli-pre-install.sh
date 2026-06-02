#!/bin/sh
set -e

# Generate the configured locale if missing; suppresses Perl locale warnings
# from the man-db dpkg trigger that fires after /usr/share/man files land.
if [ -d /usr/share/i18n/locales ] && [ -n "${LANG:-}" ] && command -v localedef >/dev/null 2>&1; then
    localedef -i "${LANG%%.*}" -f "${LANG##*.}" "${LANG}" >/dev/null 2>&1 || true
fi
