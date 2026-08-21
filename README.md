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
  automatically while a text editor or a game has the focus (Windows).
- **Tray icon** — show, hide, open the console or quit from the notification
  area (Windows).
- **Quick Access** — the managed folders are pinned to Explorer's Quick Access
  list (Windows).
- **Console** — a built in log panel showing everything the application does.

---

## Managed Folders

All paths are relative to the user's home directory:

| Folder        | Contents                                              |
| ------------- | ----------------------------------------------------- |
| `Documents`   | txt, pdf, doc(x), rtf, odt, xls(x), csv, ppt(x), md, … |
| `Images`      | jpg, png, gif, bmp, tif, webp, heic, svg, ico, …       |
| `Musics`      | mp3, wav, flac, aac, ogg, m4a, wma, opus               |
| `Videos`      | mp4, mkv, avi, mov, wmv, webm, m4v                     |
| `Executables` | exe, msi, iso, jar, bat, cmd, sh                       |
| `Archives`    | zip, rar, 7z, tar, gz, bz2, xz                         |
| `Folders`     | every directory swept out of `Downloads`               |

`Desktop` and `Downloads` are indexed and watched but never used as a target.

---

## Project Structure

```
file-organizer/
├── src/
│   ├── core/      # Scanning, classification, moving and logging
│   ├── platform/  # Hotkey, tray, Quick Access and shell integration
│   ├── ui/        # gpui views: title bar, list, console, icons
│   └── main.rs    # Application entry point and orchestration
├── assets/icons/  # SVG icons, embedded into the binary at build time
├── installer.iss  # Inno Setup script for the Windows installer
└── Cargo.toml
```

---

## Requirements

- **Rust** 1.85 or newer (the crate uses the 2024 edition)
- **Inno Setup 6** — only to build the Windows installer

Windows is the primary target. The application builds and runs on Linux and
macOS, where the file organization, the search and the console work, while the
global hotkey, the tray icon and Quick Access pinning stay inert.

---

## Build and Run

```bash
# Run in development
cargo run

# Check, lint and test
cargo check
cargo clippy
cargo test

# Release build
cargo build --release
```

The binary is written to `target/release/file-organizer`
(`file-organizer.exe` on Windows).

### Windows Installer

```batch
rem 1. Build the release binary
cargo build --release --target x86_64-pc-windows-msvc

rem 2. Package it
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" ".\installer.iss"
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
focus, so it stays out of the way.

**Nothing is pinned to Quick Access.** Pinning drives Explorer through
PowerShell; check the `[pin]` lines in the console for the failing folder.

**Files are not moved.** Only known extensions are sorted; anything else is
left untouched and reported in the console as `ignored (unknown ext)`.
