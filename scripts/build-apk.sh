#!/bin/sh
set -euo pipefail

# Build an Alpine package locally and produce a signed repo under ~/packages
# Requires: run on Alpine with root privileges (or a user in abuild group).

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

PKGNAME="baker-rust"
PKGVER="$(grep '^version' Cargo.toml | head -n1 | cut -d'=' -f2 | tr -d ' \"')"
TARBALL="$PKGNAME-$PKGVER.tar.gz"

echo "==> Preparing build environment"
apk add --no-cache alpine-sdk cargo git doas

if ! id builder >/dev/null 2>&1; then
	adduser -D builder
	addgroup builder abuild
fi

if [ ! -d /home/builder/.abuild ]; then
	su builder -c 'abuild-keygen -a -n -q'
	install -m644 /home/builder/.abuild/*.pub /etc/apk/keys/
fi

echo "==> Creating source tarball $TARBALL"
git archive --format=tar.gz --output "/tmp/$TARBALL" --prefix="$PKGNAME-$PKGVER/" HEAD

echo "==> Running abuild"
su builder -c "cd '$ROOT_DIR'; cp /tmp/$TARBALL ..; abuild checksum; abuild -r"

echo "==> Done. Packages and APKINDEX available under ~/packages/<arch>/"
