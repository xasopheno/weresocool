# Maintainer: Danny <weresocool@xasopheno.com>
pkgname=weresocool
pkgver=1.0.0
pkgrel=1
pkgdesc="***** WereSoCool __!Now In Stereo!__ ****** Make cool sounds. Impress your friends."
url="https://github.com/xasopheno/WereSoCool"
license=("GPL-3.0")
arch=("x86_64")
depends=("portaudio" "pkg-config" "lame" "vorbis-tools")
makedepends=("cargo" "portaudio" "pkg-config" "lame" "vorbis-tools")
provides=("weresocool")

pkgver() {
    (git describe --long --tags || echo "$pkgver") | sed 's/^v//;s/\([^-]*-g\)/r\1/;s/-/./g'
}

build() {
    return 0
}

package() {
    cd ..
    usrdir="$pkgdir/usr"
    mkdir -p $usrdir
    cargo install --no-track --path . --root "$usrdir"
}

