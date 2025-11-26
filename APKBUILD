# Maintainer: Arthur Hoch <arthur.j.h96@gmail.com>
pkgname=baker-rust
pkgver=0.1.0
pkgrel=0
pkgdesc="Rust implementation of BakerCM"
url="https://github.com/arthurhoch/baker-rust"
arch="x86_64 aarch64"
license="BSD-3-Clause"
depends=""
makedepends="cargo"
options="!check" # enable check() when CI env has rust+cargo available
source="$pkgname-$pkgver.tar.gz"
# Avoid colliding with the repository's own src/ directory when abuild cleans $srcdir
srcdir="$startdir/.abuild-src"
builddir="$srcdir/$pkgname-$pkgver"

prepare() {
	default_prepare
	local crate_dir="$srcdir/$pkgname-$pkgver"
	cd "$crate_dir" || return 1
	cargo fetch --manifest-path "$crate_dir/Cargo.toml"
}

build() {
	local crate_dir="$srcdir/$pkgname-$pkgver"
	cd "$crate_dir" || return 1
	cargo build --release --manifest-path "$crate_dir/Cargo.toml"
}

check() {
	# Uncomment when running tests during packaging
	local crate_dir="$srcdir/$pkgname-$pkgver"
	cd "$crate_dir" && cargo test --release --manifest-path "$crate_dir/Cargo.toml"
	:
}

package() {
	local crate_dir="$srcdir/$pkgname-$pkgver"
	cd "$crate_dir"
	install -Dm755 target/release/baker-rust "$pkgdir/usr/bin/baker-rust"
	install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}

sha512sums="SKIP"
