# File Organizer

A desktop application built with **Rust** and **gpui** that keeps your home
folder tidy: it sweeps Downloads into typed buckets and gives you a global
`Ctrl+Space` search over everything it manages.

---

## Features

- **Automatic sorting** — on startup, every top level entry in `Downloads` is
  moved into the bucket matching its extension, and folders land in `Folders`.
- **Instant search** — a searchable index of the managed folders, with a type
  icon and the containing folder for each entry.
- **Open and reveal** — click an entry to open it, right click to reveal it in
  the system file manager.
- **Live index** — the managed folders are watched, so the list follows what
  happens on disk.
- **Global hotkey** — `Ctrl+Space` toggles the window, and steps aside
  automatically while a text editor or a game has the focus.
- **Tray icon** — show, hide, open the console, toggle autostart or quit from
  the notification area.
- **Autostart** — start with the session, for the current user only.
- **Quick Access** — the managed folders are pinned to Explorer's Quick Access
  list (Windows).
- **Console** — a built in log panel showing everything the application does.

Every move is failsafe: the destination name is reserved before anything is
written, a move across file systems copies and verifies before it deletes, and
a failure at any point leaves the original where it was.

---

## Managed Folders

All paths are relative to the user's home directory:

| Folder        | Contents                                              |
| ------------- | ----------------------------------------------------- |
| `Documents`   | txt, pdf, doc(x), rtf, odt, xls(x), csv, ppt(x), md, … |
| `Pictures`    | jpg, png, gif, bmp, tif, webp, heic, svg, ico, …       |
| `Music`       | mp3, wav, flac, aac, ogg, m4a, wma, opus               |
| `Videos`      | mp4, mkv, avi, mov, wmv, webm, m4v                     |
| `Executables` | exe, msi, iso, jar, bat, cmd, sh, AppImage, deb, rpm   |
| `Archives`    | zip, rar, 7z, tar, gz, bz2, xz, zst                    |
| `Folders`     | every directory swept out of `Downloads`               |

`Desktop` and `Downloads` are indexed and watched but never used as a target.
Anything with an unrecognised extension is left exactly where it is.

The list lives in `src/core/folders.rs` and is the single source of truth: the
same set is created, swept into, indexed and watched.

---

## Project Structure

```
file-organizer/
├── src/
│   ├── core/      # Scanning, classification, moving and logging
│   ├── platform/  # Hotkey, tray, Quick Access and shell integration
│   ├── ui/        # gpui views: title bar, list, console, icons
│   └── main.rs    # Application entry point and orchestration
├── packaging/
│   ├── linux/     # .desktop entry, icon and a generic install script
│   ├── arch/      # PKGBUILD
│   └── debian/    # .deb build script and control file
├── assets/icons/  # SVG icons, embedded into the binary at build time
├── installer.iss  # Inno Setup script for the Windows installer
└── Cargo.toml
```

---

## Platform Support

| Feature              | Windows | Linux (X11)  | Linux (Wayland) |
| -------------------- | ------- | ------------ | --------------- |
| Sorting and search   | yes     | yes          | yes             |
| Tray icon            | yes     | yes          | yes             |
| Autostart            | yes     | yes          | yes             |
| Global `Ctrl+Space`  | yes     | yes          | XWayland only   |
| Focus aware muting   | yes     | yes          | XWayland only   |
| Quick Access pinning | yes     | not relevant | not relevant    |

Wayland deliberately keeps global shortcuts away from applications. Bind
`file-organizer` to a shortcut in your compositor's own settings instead; the
console says so on startup when it detects a Wayland session.

The Linux tray needs a status notifier host. KDE Plasma has one built in, and
GNOME needs the AppIndicator extension. Without one the application still runs,
just without an icon.

---

## Installing

Prebuilt binaries are attached to the
[latest release](https://github.com/unkn0wndo3s/file-organizer/releases/latest) —
that link always resolves to whatever is newest, so it stays correct across
every future version without needing to be updated here.

### Arch and Arch based

```bash
cd packaging/arch && makepkg -si
```

`makepkg` downloads the tagged source, builds it, runs the test suite and
installs the result with `pacman`. Bump `pkgver` in `PKGBUILD` to package a
release newer than the one currently pinned there.

### Debian and Ubuntu based

```bash
sudo apt install build-essential dpkg-dev pkg-config libx11-dev \
     libxkbcommon-dev libwayland-dev libfontconfig1-dev libfreetype6-dev \
     libvulkan-dev libasound2-dev
./packaging/debian/build-deb.sh
sudo apt install ./target/debian/file-organizer_*.deb
```

The version packaged is read from `Cargo.toml`, so this always builds
whatever is checked out — no version to edit by hand.

### Any other distribution

```bash
sudo ./packaging/linux/install.sh      # or --user for ~/.local
```

### Windows

Download `FileOrganizer-Setup.exe` from the
[latest release](https://github.com/unkn0wndo3s/file-organizer/releases/latest)
and run it, or build it yourself as described below.

### Starting with the session

Tick **Start with the session** in the tray menu, or run:

```bash
file-organizer --enable-autostart
file-organizer --disable-autostart
file-organizer --autostart-status
```

The setting is per user and never needs administrator rights: on Windows it is
a value under the current user's `Run` key, on Linux an XDG autostart entry.

---

## Requirements

- **Rust** 1.85 or newer (the crate uses the 2024 edition)
- **Inno Setup 6** — only to build the Windows installer

---

## Building the Binaries

Every command below has been run against this exact source tree.

```bash
# Run in development
cargo run

# Check, lint and test
cargo check
cargo clippy
cargo test
```

### Linux binary

```bash
cargo build --release
```

**Output:** `target/release/file-organizer`

### Arch package

```bash
cd packaging/arch
makepkg -f    # build only
makepkg -si   # build, and install with pacman
```

**Output:** `packaging/arch/file-organizer-<version>-1-x86_64.pkg.tar.zst`

### Debian / Ubuntu package

```bash
sudo apt install build-essential dpkg-dev pkg-config libx11-dev \
     libxkbcommon-dev libwayland-dev libfontconfig1-dev libfreetype6-dev \
     libvulkan-dev libasound2-dev
./packaging/debian/build-deb.sh
```

**Output:** `target/debian/file-organizer_<version>_<arch>.deb`

### Generic Linux install

```bash
sudo ./packaging/linux/install.sh            # system wide, into /usr/local
./packaging/linux/install.sh --user          # current user only, into ~/.local
./packaging/linux/install.sh --uninstall     # remove either of the above
```

Builds the release binary itself; no separate `cargo build` needed first.

### Windows binary

Build **on Windows**, with the MSVC toolchain — cross-compiling gpui's
graphics stack from Linux is not a supported path:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

**Output:** `target\x86_64-pc-windows-msvc\release\file-organizer.exe`

### Windows installer

After the step above, with [Inno Setup 6](https://jrsoftware.org/isinfo.php)
installed:

```powershell
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" .\installer.iss
```

**Output:** `FileOrganizer-Setup.exe`

---

## Usage

Press `Ctrl+Space` to toggle the window, type to filter, click an entry to open
it and right click to reveal it. The gear button in the title bar, and the
`Console` entry of the tray menu, open the log panel.

---

## Troubleshooting

**The hotkey does not respond.** Another application may already own
`Ctrl+Space`. The console reports the registration result on startup. The
shortcut is also released on purpose while a text editor or a game has the
focus, so it stays out of the way. Under Wayland it only reaches XWayland
windows; bind it in your compositor instead.

**No tray icon on Linux.** The desktop needs a status notifier host. On GNOME,
install the AppIndicator extension; elsewhere, install
`libappindicator-gtk3` (Arch) or `libayatana-appindicator3-1` (Debian).

**Nothing is pinned to Quick Access.** Pinning drives Explorer through
PowerShell; check the `[pin]` lines in the console for the failing folder.

**Files are not moved.** Only known extensions are sorted; anything else is
left untouched and reported in the console as `ignored (unknown ext)`.
