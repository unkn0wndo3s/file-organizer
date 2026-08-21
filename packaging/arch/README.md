# Arch Linux package

## Building and installing

```bash
cd packaging/arch
makepkg -si
```

`makepkg` pulls the build dependencies, compiles the release binary, runs the
test suite and installs the resulting package with `pacman`.

## Building from a local checkout

The `PKGBUILD` fetches the tagged release tarball. To package the working tree
instead, point `source` at it:

```bash
makepkg -si --skipchecksums \
    SRCDEST=.. source="file-organizer-2.0.0.tar.gz::file://$(cd ../.. && pwd)"
```

## Starting with the session

The package does not enable autostart for every user on the machine. Each user
turns it on for themselves:

```bash
file-organizer --enable-autostart
```

or ticks **Start with the session** in the tray menu.

## Tray icon

Most Arch desktops (KDE Plasma, and GNOME with the AppIndicator extension)
provide a status notifier host out of the box. On a desktop that does not,
install the optional dependency:

```bash
sudo pacman -S libappindicator-gtk3
```

## Note on the license

The project is released under CC BY-NC-SA 4.0. The NonCommercial clause makes
it ineligible for the official Arch repositories and the AUR's `license`
conventions treat it as `custom:`, so this `PKGBUILD` is meant for local and
private builds.
