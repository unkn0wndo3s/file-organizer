# Release Notes - File Organizer v2.0.0

**Release Date:** August 21, 2026

## 🚀 Major Release: Full Rewrite in Rust

File Organizer has been completely rewritten from JavaFX/Java to **Rust**
with **gpui**, and now runs natively on **Windows and Linux** (Arch based and
Debian/Ubuntu based distributions).

---

## Added

- **Linux support**: full feature parity with Windows, including the global
  hotkey, tray icon, focus aware muting and autostart
  - Tray icon via a StatusNotifierItem (D-Bus), working out of the box on
    KDE Plasma and on GNOME with the AppIndicator extension
  - Global `Ctrl+Space` via an X11 key grab (XWayland windows only under
    Wayland, since Wayland has no global shortcut API)
  - Focus detection via `_NET_ACTIVE_WINDOW` / `WM_CLASS`
- **Autostart**: start with the user's session on both platforms, toggled
  from the tray menu or the command line
  - `file-organizer --enable-autostart`
  - `file-organizer --disable-autostart`
  - `file-organizer --autostart-status`
- **Distribution packaging**:
  - `PKGBUILD` for Arch and Arch based distributions
  - `.deb` build script for Debian and Ubuntu based distributions
  - A generic POSIX install script for any other distribution
- **Failsafe file moves**: every move reserves its destination name before
  writing, verifies a cross device copy against the source before deleting
  it, and leaves the original untouched on any failure
- **Command line interface**: `--version`, `--help`, and the autostart flags
  above

## Changed

- **Complete platform rewrite**: JavaFX, Maven and JNA replaced by Rust,
  Cargo, gpui/gpui-component and direct Win32 / X11 bindings
- **Single source of truth for managed folders**: the sweep destination, the
  folder initializer and the search index now read from one list, instead of
  two lists that had drifted apart
- **Virtualized, indexed search**: the result list renders only its visible
  rows and matches against a precomputed, lowercased index, so it stays fast
  regardless of how many files are managed
- **Event driven background work**: the application now sleeps until a file
  system event, a log line or a command actually arrives, instead of polling
  every 200ms
- **Icons embedded into the binary**: no external assets folder to ship or
  lose track of

## Fixed

- Images and audio files were being sorted into `Images` and `Musics`, folders
  the search index never watched; both now share the same `Pictures` and
  `Music` folders the index uses
- The managed folders were only watched if they already existed before the
  first startup sweep ran; they are now created before the window opens
- A move interrupted partway through could previously leave a partial copy
  behind or delete the original before the copy was verified

## Removed

- The Java/JavaFX/Maven implementation (`core`, `ui`, `windows`, `app`
  modules, and the parent `pom.xml`)
- An unused dry run planning API (`FilePlanner`, `RuleSet`,
  `PreArrangeService` and their Rust equivalents) that was never reachable
  from the entry point in either implementation

---

## Platform Support

| Feature               | Windows | Linux (X11) | Linux (Wayland) |
| --------------------- | ------- | ----------- | ---------------- |
| Sorting and search    | yes     | yes         | yes              |
| Tray icon              | yes     | yes         | yes              |
| Autostart              | yes     | yes         | yes              |
| Global `Ctrl+Space`   | yes     | yes         | XWayland only    |
| Focus aware muting    | yes     | yes         | XWayland only    |
| Quick Access pinning  | yes     | n/a         | n/a              |

---

## Upgrading

This is a full rewrite: uninstall the previous JavaFX based build before
installing v2.0.0. Settings are not carried over — there were none to carry;
the managed folders and their contents are untouched.

---

## Previous Releases

### v1.1.2 - Smart Hotkey Management

- Hotkey deactivation based on the focused application (editors, games)
- Browser exception, so Ctrl+Space kept working in the browser
- Improved window classification and focus detection

### v1.1.0 - Hotkey Fixes

- Fixed hotkeys not working in the compiled executable
- Added proper module dependencies for jpackage
- Enhanced JVM options for better compatibility

### v1.0.0 - Initial Release

- Global hotkey support (Ctrl+Space)
- Automatic file organization from the Downloads folder
- Real-time file search across organized folders
- Windows integration and system tray support

---

**Developer:** Unkn0wndo3s
**License:** Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)
