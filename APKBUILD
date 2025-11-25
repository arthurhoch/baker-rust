# Maintainer: Your Name <you@example.com>
pkgname=baker-rust
pkgver=0.1.0
pkgrel=0
pkgdesc="Rust implementation of BakerCM"
url="https://github.com/$(whoami)/baker-rust"
arch="x86_64 aarch64"
license="BSD-3-Clause"
depends=""
makedepends="cargo"
options="!check" # enable check() when CI env has rust+cargo available
source="$pkgname-$pkgver.tar.gz"
builddir="$srcdir/$pkgname-$pkgver"

prepare() {
	default_prepare
	cargo fetch
}

build() {
	cargo build --release
}

check() {
	# Uncomment when running tests during packaging
	# cargo test --release
	:
}

package() {
	install -Dm755 target/release/baker-rust "$pkgdir/usr/bin/baker-rust"
	install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}

sha512sums="SKIP"
