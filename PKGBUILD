# Copyright (C) 2025 MaskedRedstonerProZ
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

pkgname=clip-keeper
pkgver=1.1.0
pkgrel=1
pkgdesc="Very simple rofi frontend for pass"
url="https://gitlab.com/MaskedRedstonerProZ/${pkgname}.git"
source=(
	"https://gitlab.com/MaskedRedstonerProZ/${pkgname}/-/raw/master/install"
	"https://gitlab.com/MaskedRedstonerProZ/${pkgname}/-/raw/master/LICENSE"
)
arch=("x86_64")
license=("GPL-3.0-or-later")
makedepends=("wget")
depends=("rofi" "pass")
optdepends=()
sha256sums=('SKIP' 'SKIP')

build() {
	exec ${PWD}/install
}

package() {

	install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
