# Release Notes - File Organizer v2.1.0

**Release Date:** September 4, 2026

## Highlights: A Real Launcher, Not a Window That Happens to Search

The window used to open maximized on every launch and stay a fixed size for
its whole life. It now behaves like an actual launcher: hidden until asked
for, sized like a spotlight popup on whichever screen you're working on, and
dismissed the same way it's brought up.

---

## Added

- **Starts hidden in the tray**: nothing appears on launch; bring the window
  up with `Ctrl+Space`, the tray icon, or the new `--toggle` flag
- **`--toggle` command**: shows or hides an already running instance from the
  command line, so the window can be bound to a key in a compositor or
  window manager the global hotkey cannot reach (Wayland compositors without
  an XWayland-reachable focused window, in particular)
- **Launcher style window**: fixed at 2/5 of the screen's width and five
  results tall instead of filling the screen, translucent, with no close or
  minimize button
- **Follows the pointer**: every time the window is shown, it resizes and
  recenters onto whichever display currently has the pointer, instead of
  staying wherever - and however large - it was first created
- **Escape hides the window**
- **Every managed folder is kept tidy, not just `Downloads`**: a file dragged
  in by hand or saved directly into the wrong bucket is now relocated on
  startup too; subfolders are left exactly where they were put, as before
- The Arch package (`packaging/arch/*.pkg.tar.zst`) is now built and attached
  to every release automatically, alongside the Linux binary/`.deb` and the
  Windows binary/installer

## Changed

- **Runs through XWayland on Linux** whenever an X server is reachable,
  instead of gpui's native Wayland backend: Wayland's `xdg-shell` has no
  request to restore a minimized window, which made hiding to the tray a
  dead end, and running through XWayland also lets the existing `Ctrl+Space`
  X11 grab reach the window on more Wayland compositors than before
- **Console panel replaces the search bar and result list** instead of
  sharing the window with them, and now renders only its visible lines
  instead of all of them on every redraw — the panel used to stall the
  window briefly on every mouse movement once open
- Title bar text and the settings icon are now a visible white
- The Windows leg of CI now builds on an actual Windows runner instead of
  cross-compiling from Linux, which turned out to be a dead end: gpui needs
  `fxc.exe` from the proprietary Windows SDK to compile its Direct3D
  shaders, and the open source alternative only emits a format gpui's D3D11
  device cannot load

## Fixed

- `Ctrl+Space`, the tray, and `--toggle` all toggled the window by comparing
  against `is_window_active()`, which reports a visible-but-unfocused window
  (the user simply clicked elsewhere) the same as a minimized one — a second
  press re-showed it instead of hiding it. The app now tracks its own
  shown/hidden state instead
- The tray's **Console** entry only toggled the panel's own visibility,
  invisible while the window was hidden in the tray; it now shows the window
  too
- A missing `libxkbcommon-x11` dependency broke the Linux build in any fresh
  or minimal environment - CI, both README install snippets, the `.deb`'s
  `control` file and the Arch `PKGBUILD` all listed `libxkbcommon` but not
  the separate `-x11` package actually being linked against
- The window could open at 0×0 on Wayland, since a client only learns about
  its displays asynchronously after connecting; startup now waits briefly
  for one to be reported before sizing the window
- Requesting a maximized window together with a minimum size at creation
  sent KWin an invalid zero-size viewport during the maximize handshake

---

## Platform Support

| Feature              | Windows | Linux (X11)  | Linux (Wayland) |
| -------------------- | ------- | ------------ | --------------- |
| Sorting and search   | yes     | yes          | yes             |
| Tray icon            | yes     | yes          | yes             |
| Autostart            | yes     | yes          | yes             |
| Global `Ctrl+Space`  | yes     | yes          | best effort\*   |
| `--toggle` command   | yes     | yes          | yes             |
| Focus aware muting   | yes     | yes          | best effort\*   |
| Quick Access pinning | yes     | not relevant | not relevant    |

\* On Wayland, `file-organizer` runs through XWayland instead of natively
whenever an X server is reachable. That fixes the window's hide/restore
round trip and lets `Ctrl+Space` reach the window while any XWayland client
has focus, but a Wayland compositor can still route key presses straight to
a focused native Wayland window without forwarding them to XWayland at all,
so the shortcut can still be silent depending on what's focused.
`file-organizer --toggle` is unaffected by any of this — bind it to a key in
your compositor's own shortcut settings for a toggle that always works.

---

## Upgrading

No settings to migrate, and the managed folders and their contents are
untouched. The window is now a small popup instead of a maximized one on
launch - if you relied on the old always-maximized-on-launch behavior for
anything, that's gone in favor of the launcher style window described above.

---

## Previous Releases

### v2.0.0 - Full Rewrite in Rust

- Complete rewrite from JavaFX/Java to Rust with gpui
- Full Linux support: tray icon, global hotkey, focus aware muting, autostart
- Failsafe file moves, virtualized/indexed search, event driven background work

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
