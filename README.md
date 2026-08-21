# File Organizer

A desktop application built with **Rust** and **gpui** that keeps your home
folder tidy: it sweeps Downloads into typed buckets and gives you a global
`Ctrl+Space` search over everything it manages.

Runs on Windows and Linux (Arch and Debian/Ubuntu based distributions, or any
other distribution through a generic install path).

## Contents

- [Features](#features)
- [Managed Folders](#managed-folders)
- [Project Structure](#project-structure)
- [Platform Support](#platform-support)
- [Installing](#installing)
- [Requirements](#requirements)
- [Building the Binaries](#building-the-binaries)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

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
│   ├── core/      # Platform independent logic: no gpui, no OS APIs
│   ├── platform/  # OS integration: hotkey, tray, focus, autostart, shell
│   ├── ui/        # gpui views: title bar, list, console, icons
│   └── main.rs    # Wires core, platform and ui together
├── packaging/
│   ├── linux/     # .desktop entry, icon and a generic install script
│   ├── arch/      # PKGBUILD
│   └── debian/    # .deb build script and control file
├── assets/icons/  # SVG icons, embedded into the binary at build time
├── .github/workflows/  # CI: tests every push, builds every release asset
├── installer.iss  # Inno Setup script for the Windows installer
└── Cargo.toml
```

### `src/core` — what happens, independent of the OS or the UI

No `gpui`, no `windows-sys`, no `x11rb`; everything here is plain Rust that
`cargo test` can exercise without a display. This is where to look first for
sorting rules or move behavior.

| Module | Responsibility |
| --- | --- |
| `folders.rs` | The single source of truth: which folders are managed, and which file extension goes into which. Creates them (`ensure_all`), and is read by both the mover and the index — see [Managed Folders](#managed-folders). |
| `scanner.rs` | Lists the visible top level entries of the managed folders. |
| `mover.rs` | Moves an entry into its bucket. Every move is failsafe: see [Features](#features). |
| `fs_util.rs` | Small path helpers: extension extraction, hidden file detection, unique name generation. |
| `log_bus.rs` | A process wide broadcast channel: any part of the app calls `log_bus::log(...)`, and every registered listener (stdout, the in-app console) receives it. |
| `model.rs` | The plain data types (`FsEntry`) passed between the modules above. |

### `src/platform` — the same job, four different ways

Every file here has a `#[cfg(windows)]` branch, a `#[cfg(target_os = "linux")]`
branch, and usually a fallback for anything else, all behind one shared
interface so `main.rs` never branches on OS itself.

| Module | Windows | Linux |
| --- | --- | --- |
| `hotkey.rs` | `RegisterHotKey` / a Win32 message loop | `XGrabKey` on the root window (X11; XWayland only under Wayland) |
| `tray.rs` | A Win32 tray icon (`Shell_NotifyIcon`) | A StatusNotifierItem over D-Bus (`ksni`) |
| `focus_monitor.rs` | Polls `GetForegroundWindow` | Polls `_NET_ACTIVE_WINDOW` / `WM_CLASS` over X11 |
| `autostart.rs` | A value under the user's registry `Run` key | An XDG autostart `.desktop` entry |
| `quick_access.rs` | Pins folders to Explorer via PowerShell | n/a |
| `shell.rs` | Open/reveal a path with the OS file manager | Open/reveal a path with the OS file manager |

### `src/ui` — what you see

Built on [`gpui`](https://github.com/zed-industries/zed/tree/main/crates/gpui)
(Zed's UI framework) and
[`gpui-component`](https://github.com/longbridge/gpui-component). `list.rs` is
the searchable index view — it renders through `uniform_list`, so only the
rows on screen are ever built, regardless of how many files are managed.

### How `main.rs` wires it together

The hotkey, the tray and the file watcher each run on their own thread and
none of them may touch a `gpui` entity directly — only the thread that owns
the window can. `main.rs` bridges this with `async-channel`: each background
thread sends onto a channel, and one `cx.spawn` per channel awaits it and
applies the update on the UI thread. Nothing polls on a timer; the app is
fully idle until an event, a log line or a command actually arrives.

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

Build **on Windows**, with the MSVC toolchain:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

**Output:** `target\x86_64-pc-windows-msvc\release\file-organizer.exe`

Cross-compiling this from Linux is not possible, not just unsupported: gpui's
Direct3D 11 renderer compiles its HLSL shaders to DXBC through `fxc.exe`, part
of the proprietary Windows SDK. The open source alternative, `dxc`, takes the
same command line but only emits DXIL (the Shader Model 6 / D3D12 format) —
tested against gpui's own shaders, it builds without error and produces a
binary a D3D11 device cannot load. There is no portable, redistributable tool
that emits DXBC outside of a real Windows SDK install.

`.github/workflows/release.yml` builds this leg on a `windows-latest` GitHub
Actions runner, so cutting a release never actually requires a Windows
machine — push a `v*` tag from anywhere and it, the Linux binary and the
`.deb` all build and attach themselves to that tag's release.

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

---

## Contributing

Forking and modifying the source is welcome under the terms of the
[license](#license) below.

1. Fork the repository and clone your fork.
2. Create a branch for your change.
3. Make the change. `src/core` is plain Rust and the easiest place to start —
   see [Project Structure](#project-structure) for what lives where.
4. Before opening a pull request:

   ```bash
   cargo fmt
   cargo clippy --all-targets   # this repository builds with zero warnings
   cargo test                   # unit tests only, nothing touches your real home folder
   ```

   Tests that exercise the file mover run against a temporary directory under
   `std::env::temp_dir()`, never against `dirs::home_dir()`, so running the
   suite is always safe.
5. Open a pull request describing what changed and why.

### Commit messages

This repository uses [Conventional Commits](https://www.conventionalcommits.org/):
`type(scope): summary`, where `type` is one of `feat`, `fix`, `docs`, `chore`,
`perf`, `ci` or `refactor`, and `scope` is the module touched (`core`,
`platform`, `ui`, `packaging`...). Look at `git log` for examples. Keep the
summary in the imperative mood ("add", not "added"), and explain the *why* in
the body when it is not obvious from the diff.

### Code style

- Default to no comments. Add one only when the *why* is not obvious from the
  code itself — a platform quirk, a workaround, an invariant a reader could
  otherwise break by accident.
- No unstable Rust features; the crate targets stable, edition 2024 (Rust
  1.85+).
- New behavior in `src/core` should come with a unit test in the same file,
  in a `#[cfg(test)] mod tests` block at the bottom — that is the existing
  pattern throughout the module.
- `src/platform` code should keep the Windows, Linux and fallback
  implementations behind the same function signatures, as `hotkey.rs` and
  `tray.rs` already do, so `main.rs` never has to know which platform it is
  running on.

### Reporting a bug or requesting a feature

Open an [issue](https://github.com/unkn0wndo3s/file-organizer/issues). For a
bug, include your OS, desktop environment (on Linux), and the relevant lines
from the in-app console — the gear icon in the title bar, or **Console** in
the tray menu, opens it.

---

## License

[Attribution-NonCommercial-ShareAlike 4.0 International](LICENSE) (CC BY-NC-SA 4.0).

In plain terms, and without this being legal advice:

- **You can** read, run, study, modify and share this source, and your own
  modified versions of it, for free.
- **You can't** sell it, or use it — modified or not — as part of a paid
  product or service, without the author's separate permission.
- **If you share a modified version**, it has to carry the same license and
  credit the original author.

This is a stricter license than most software on GitHub uses, and it is why
the project cannot be submitted to the official Arch repositories, the AUR,
or Debian main, all of which require a license without a NonCommercial
clause — see `packaging/arch/README.md` for the detail. The packaging in this
repository is meant for building and installing the project yourself, not for
redistribution through those channels.

---

## Acknowledgments

Built with [Rust](https://www.rust-lang.org/) and
[gpui](https://github.com/zed-industries/zed/tree/main/crates/gpui), the UI
framework behind [Zed](https://zed.dev/), together with
[gpui-component](https://github.com/longbridge/gpui-component) for its input,
icon and list widgets.

Platform integration relies on
[windows-sys](https://github.com/microsoft/windows-rs) on Windows, and on
[x11rb](https://github.com/psychon/x11rb) and
[ksni](https://github.com/iovxw/ksni) on Linux, plus
[notify](https://github.com/notify-rs/notify) for watching the managed
folders on both.

**Developer:** Unkn0wndo3s
