#!/usr/bin/env sh
# Builds and installs File Organizer on any Linux distribution.
#
#   sudo ./packaging/linux/install.sh            install system wide
#        ./packaging/linux/install.sh --user     install for the current user
#   sudo ./packaging/linux/install.sh --uninstall
#
# Distribution packages are preferred where they exist: see packaging/arch and
# packaging/debian.

set -eu

REPO=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
NAME=file-organizer
PREFIX=/usr/local
MODE=install
USER_SCOPE=no

for argument in "$@"; do
    case "$argument" in
        --user) USER_SCOPE=yes ;;
        --uninstall) MODE=uninstall ;;
        --prefix=*) PREFIX=${argument#--prefix=} ;;
        *) echo "$0: unknown option $argument" >&2; exit 2 ;;
    esac
done

if [ "$USER_SCOPE" = yes ]; then
    PREFIX=${XDG_DATA_HOME:-$HOME/.local}
    BIN_DIR=$HOME/.local/bin
    DATA_DIR=${XDG_DATA_HOME:-$HOME/.local/share}
else
    BIN_DIR=$PREFIX/bin
    DATA_DIR=$PREFIX/share
fi

DESKTOP_DIR=$DATA_DIR/applications
ICON_DIR=$DATA_DIR/icons/hicolor/scalable/apps

if [ "$MODE" = uninstall ]; then
    # Leave the user's own autostart entry alone; only the shipped files go.
    rm -f "$BIN_DIR/$NAME" "$DESKTOP_DIR/$NAME.desktop" "$ICON_DIR/$NAME.svg"
    [ -x "$(command -v update-desktop-database)" ] && update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    echo "Removed $NAME."
    exit 0
fi

if [ "$USER_SCOPE" = no ] && [ "$(id -u)" -ne 0 ]; then
    echo "$0: a system wide install needs root; re-run with sudo, or pass --user" >&2
    exit 1
fi

echo "Building the release binary..."
( cd "$REPO" && cargo build --release --locked )

echo "Installing to $BIN_DIR..."
install -Dm755 "$REPO/target/release/$NAME" "$BIN_DIR/$NAME"
install -Dm644 "$REPO/packaging/linux/$NAME.desktop" "$DESKTOP_DIR/$NAME.desktop"
install -Dm644 "$REPO/packaging/linux/$NAME.svg" "$ICON_DIR/$NAME.svg"

[ -x "$(command -v update-desktop-database)" ] && update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
[ -x "$(command -v gtk-update-icon-cache)" ] && gtk-update-icon-cache -q "$DATA_DIR/icons/hicolor" 2>/dev/null || true

echo
echo "Installed $NAME to $BIN_DIR/$NAME."
echo "To start it with your session, run:  $NAME --enable-autostart"
