#!/usr/bin/env bash
# Builds a .deb for Debian and Ubuntu based distributions.
#
#   ./packaging/debian/build-deb.sh          build for the host architecture
#   sudo apt install ./target/debian/*.deb   install the result
#
# Build requirements: cargo, dpkg-deb, and the development packages listed in
# packaging/debian/README.md.

set -euo pipefail

REPO=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
NAME=file-organizer
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO/Cargo.toml" | head -1)
ARCH=$(dpkg --print-architecture)
STAGE=$REPO/target/debian/$NAME
OUTPUT=$REPO/target/debian/${NAME}_${VERSION}_${ARCH}.deb

echo "Building $NAME $VERSION for $ARCH..."
cargo build --release --locked --manifest-path "$REPO/Cargo.toml"

rm -rf "$STAGE"
install -Dm755 "$REPO/target/release/$NAME"          "$STAGE/usr/bin/$NAME"
install -Dm644 "$REPO/packaging/linux/$NAME.desktop" "$STAGE/usr/share/applications/$NAME.desktop"
install -Dm644 "$REPO/packaging/linux/$NAME.svg"     "$STAGE/usr/share/icons/hicolor/scalable/apps/$NAME.svg"
install -Dm644 "$REPO/LICENSE"                       "$STAGE/usr/share/doc/$NAME/copyright"
install -Dm644 "$REPO/README.md"                     "$STAGE/usr/share/doc/$NAME/README.md"

# dpkg reports Installed-Size in kibibytes.
INSTALLED_SIZE=$(du -ks "$STAGE" | cut -f1)

mkdir -p "$STAGE/DEBIAN"
sed -e "s/@VERSION@/$VERSION/" \
    -e "s/@ARCH@/$ARCH/" \
    -e "s/@INSTALLED_SIZE@/$INSTALLED_SIZE/" \
    "$REPO/packaging/debian/control.in" > "$STAGE/DEBIAN/control"

install -Dm755 "$REPO/packaging/debian/postinst" "$STAGE/DEBIAN/postinst"
install -Dm755 "$REPO/packaging/debian/prerm"    "$STAGE/DEBIAN/prerm"

dpkg-deb --root-owner-group --build "$STAGE" "$OUTPUT"

echo
echo "Built $OUTPUT"
echo "Install it with:  sudo apt install $OUTPUT"
