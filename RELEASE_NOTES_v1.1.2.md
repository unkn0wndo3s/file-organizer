# Release Notes - File Organizer v1.1.2

**Release Date:** October 8, 2025

## 🔧 Patch Notes

### Fixed

- Hotkey not being deactivated in text editors (Cursor, VS Code, etc.)
- Hotkey remaining active in games when it should be disabled
- Focus monitoring not properly detecting application types
- Hotkey synchronization issues between internal state and Windows API

### Changed

- **Smart Hotkey Management**: Hotkey now intelligently deactivates based on focused application
- **Enhanced Focus Detection**: Improved window classification for editors, games, and browsers
- **Priority-based Detection**: Title-based detection takes priority over window class detection
- **Browser Exception**: Browsers (Chrome, Firefox, Edge, etc.) now keep hotkey active
- **All logs and comments translated to English** for better international support

### Added

- **Focus Monitoring System**: Real-time detection of foreground window changes
- **Application Classification**: Automatic detection of text editors, games, and browsers
- **Electron Support**: Proper detection of VS Code, Cursor, and other Electron-based editors
- **Game Detection**: Comprehensive game detection for popular gaming platforms
- **Browser Whitelist**: Explicit browser detection to prevent hotkey deactivation
- **Debug Logging**: Enhanced debug output for troubleshooting focus detection

### Technical Details

- **Window Class Detection**: Support for 20+ text editor window classes
- **Game Detection**: Support for Unity, Unreal, Source, DirectX, OpenGL, and 100+ game titles
- **Browser Detection**: Support for Chrome, Firefox, Edge, Safari, Opera, and other browsers
- **Error Handling**: Robust error handling for Windows API calls (error codes 1409, 1419)
- **Thread Management**: Separate threads for hotkey monitoring and focus detection

### Behavior Changes

- **Text Editors**: Hotkey automatically deactivated (Ctrl+Space freed for editor use)
- **Games**: Hotkey automatically deactivated (Ctrl+Space freed for game controls)
- **Browsers**: Hotkey remains active (Ctrl+Space opens File Organizer interface)
- **Other Apps**: Hotkey remains active (Ctrl+Space opens File Organizer interface)

---

## Previous Releases

### v1.1.0 - Hotkey Fixes

- Fixed hotkeys not working in compiled executable
- Added proper module dependencies for jpackage
- Enhanced JVM options for better compatibility

### v1.0.0 - Initial Release

- Global hotkey support (Ctrl+Space, Alt+F12, Win+G, F11, F12)
- Automatic file organization from Downloads folder
- Real-time file search across organized folders
- Windows integration and system tray support

---

**Developer:** Unkn0wndo3s  
**License:** Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)
