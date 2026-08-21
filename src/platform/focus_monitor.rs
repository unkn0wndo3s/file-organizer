//! Watches the foreground window so the global hotkey can step aside while the
//! user is typing in an editor or playing a game.

use crate::core::log_bus;
use std::sync::Arc;

/// Window classes commonly used by text editors and IDEs.
const TEXT_EDITOR_CLASSES: &[&str] = &[
    "Notepad",
    "Vim",
    "VimGtk",
    "Code",
    "Chrome_WidgetWin_1",
    "Chrome_WidgetWin_0",
    "SunAwtFrame",
    "SWT_Window0",
    "ConsoleWindowClass",
    "Edit",
    "RichEdit20W",
    "RICHEDIT50W",
    "Scintilla",
    "wxWindowClassNR",
    "TkTopLevel",
    "Qt5QWindowIcon",
    "Qt6QWindowIcon",
    "CefBrowserWindow",
    "ElectronMainWindow",
    "Chrome_RenderWidgetHostHWND",
    "Chrome_WidgetWin_2",
];

/// Window classes commonly used by game engines.
const GAME_CLASSES: &[&str] = &[
    "UnityWndClass",
    "UnrealWindow",
    "Valve001",
    "TankWindowClass",
    "CryEngine",
    "Frostbite",
    "D3DWindow",
    "OpenGLWindow",
    "SDL_app",
    "AllegroWindow",
    "SFML_Window",
    "GLFW3",
    "SDL_Window",
    "GameWindow",
    "MainWindow",
    "RenderWindow",
    "DirectXWindow",
    "VulkanWindow",
    "MetalWindow",
    "GameEngine",
];

/// Title fragments that identify a text editor.
const TEXT_EDITOR_KEYWORDS: &[&str] = &[
    "notepad", "notepad++", "vim", "emacs", "sublime", "atom", "code", "cursor",
    "visual studio", "intellij", "eclipse", "netbeans", "textpad", "editplus",
    "ultraedit", "textmate", "gedit", "kate", "mousepad", "leafpad", "geany",
    "bluefish", "komodo", "brackets", "light table", "zed", "nova", "coda",
    "textwrangler", "bbedit", "smultron", "textastic", "ia writer", "ulysses",
    "bear", "typora", "mark text", "zettlr", "obsidian", "logseq", "roam",
    "notion", "evernote", "onenote", "word", "pages", "libreoffice writer",
    "openoffice writer", "abiwriter", "kword", "calligra words", "focuswriter",
    "jarte", "writemonkey", "q10", "dark room", "write room", "ommwriter",
    "manuskript", "ywriter", "scrivener", "storyist", "celtx", "final draft",
    "fade in", "highland", "writerduet", "trelby", "kit scenarist", "kitscenarist",
];

/// Title fragments that identify a game or a game launcher.
const GAME_KEYWORDS: &[&str] = &[
    "steam", "epic games", "origin", "uplay", "battle.net", "gog", "itch.io",
    "minecraft", "fortnite", "league of legends", "dota", "counter-strike", "cs:go",
    "valorant", "apex legends", "call of duty", "battlefield", "fifa", "pes",
    "world of warcraft", "final fantasy", "elder scrolls", "skyrim", "fallout",
    "grand theft auto", "gta", "assassin's creed", "witcher", "cyberpunk",
    "red dead redemption", "god of war", "spider-man", "batman", "tomb raider",
    "resident evil", "silent hill", "metal gear", "halo", "gears of war",
    "uncharted", "last of us", "horizon", "ghost of tsushima", "bloodborne",
    "dark souls", "sekiro", "elden ring", "monster hunter", "street fighter",
    "tekken", "mortal kombat", "smash bros", "mario", "zelda", "pokemon",
    "animal crossing", "sims", "simcity", "cities skylines", "civilization",
    "age of empires", "command & conquer", "starcraft", "warcraft", "diablo",
    "path of exile", "destiny", "anthem", "division", "borderlands",
    "mass effect", "dragon age", "baldur's gate", "pillars of eternity",
    "divinity", "wasteland", "xcom", "fire emblem", "persona", "yakuza",
    "nier", "bayonetta", "devil may cry", "darksiders",
    "doom", "quake", "wolfenstein", "prey", "dishonored", "bioshock",
    "half-life", "portal", "left 4 dead", "team fortress", "dota 2",
    "overwatch", "paladins", "smite", "heroes of the storm", "hots",
    "world of tanks", "war thunder", "crossout", "armored warfare",
    "world of warships", "eve online", "elite dangerous", "star citizen",
    "no man's sky", "subnautica", "terraria", "stardew valley", "factorio",
    "satisfactory", "oxygen not included", "rimworld", "kenshi", "mount & blade",
    "total war", "crusader kings", "europa universalis", "hearts of iron",
    "stellaris", "endless space", "galactic civilizations", "master of orion",
    "x4", "everspace", "rebel galaxy", "freespace", "wing commander",
    "tie fighter", "x-wing", "star wars", "star trek", "battlestar galactica",
    "game", "gaming", "play", "playing", "launcher", "launcher.exe",
];

/// Title fragments that identify a browser, which must keep the hotkey available.
const BROWSER_KEYWORDS: &[&str] = &[
    "chrome", "firefox", "edge", "safari", "opera", "vivaldi", "brave",
    "tor browser", "waterfox", "pale moon", "seamonkey", "internet explorer",
    "microsoft edge", "google chrome", "mozilla firefox",
    "browser", "web browser", "navigateur", "navigateur web",
];

/// Invoked whenever the foreground window changes.
pub type FocusChangeCallback = Arc<dyn Fn(FocusVerdict) + Send + Sync + 'static>;

/// What the newly focused window means for the global shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusVerdict {
    /// The window is an editor or a game; give the shortcut back to it.
    ReleaseHotkey,
    /// An ordinary window; the shortcut stays ours.
    KeepHotkey,
}

impl FocusVerdict {
    fn of(title: &str, class_name: &str) -> Self {
        if should_disable_hotkey(title, class_name) {
            Self::ReleaseHotkey
        } else {
            Self::KeepHotkey
        }
    }
}

/// Decides whether the hotkey should be released while `title` / `class_name`
/// holds the focus. Browsers keep the hotkey even when they share a window
/// class with an Electron based editor.
pub fn should_disable_hotkey(title: &str, class_name: &str) -> bool {
    let title_lower = title.to_lowercase();
    let class_lower = class_name.to_lowercase();

    let contains_any = |haystack: &str, needles: &[&str]| needles.iter().any(|needle| haystack.contains(needle));

    if contains_any(&title_lower, TEXT_EDITOR_KEYWORDS) || contains_any(&title_lower, GAME_KEYWORDS) {
        return true;
    }

    let looks_like_a_browser = contains_any(&title_lower, BROWSER_KEYWORDS);
    if !looks_like_a_browser
        && TEXT_EDITOR_CLASSES.iter().any(|class| class_lower.contains(&class.to_lowercase()))
    {
        return true;
    }

    if GAME_CLASSES.iter().any(|class| class_lower.contains(&class.to_lowercase())) {
        return true;
    }

    false
}

/// Polls the foreground window and reports every change.
pub struct FocusMonitor {
    callback: FocusChangeCallback,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl FocusMonitor {
    pub fn new(callback: FocusChangeCallback) -> Self {
        Self { callback, running: Arc::new(std::sync::atomic::AtomicBool::new(false)) }
    }

    /// Starts the polling thread. Calling it twice is a no-op.
    pub fn start(&self) {
        use std::sync::atomic::Ordering;

        if self.running.swap(true, Ordering::SeqCst) {
            log_bus::log("[focus] monitor already started");
            return;
        }

        log_bus::log("[focus] starting focus monitoring...");
        imp::spawn(Arc::clone(&self.callback), Arc::clone(&self.running));
    }

    /// Asks the polling thread to stop at its next iteration.
    pub fn stop(&self) {
        use std::sync::atomic::Ordering;

        if self.running.swap(false, Ordering::SeqCst) {
            log_bus::log("[focus] stopping focus monitoring...");
        }
    }
}

impl Drop for FocusMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod imp {
    use super::{FocusChangeCallback, FocusVerdict};
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    };

    pub fn spawn(callback: FocusChangeCallback, running: Arc<AtomicBool>) {
        std::thread::Builder::new()
            .name("focus-monitor".to_string())
            .spawn(move || {
                log_bus::log("[focus] monitoring thread started");
                let mut last_window: HWND = std::ptr::null_mut();

                while running.load(Ordering::SeqCst) {
                    let current = unsafe { GetForegroundWindow() };

                    if !current.is_null() && current != last_window {
                        last_window = current;

                        let title = window_text(current);
                        let class_name = window_class(current);
                        let verdict = FocusVerdict::of(&title, &class_name);

                        log_bus::log(format!("[focus] focus moved to: {title} ({class_name}) -> {verdict:?}"));
                        callback(verdict);
                    }

                    std::thread::sleep(Duration::from_millis(100));
                }

                log_bus::log("[focus] monitoring thread finished");
            })
            .expect("focus monitor thread can be spawned");
    }

    fn window_text(window: HWND) -> String {
        let length = unsafe { GetWindowTextLengthW(window) };
        if length <= 0 {
            return String::new();
        }

        let mut buffer = vec![0u16; length as usize + 1];
        let copied = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
        from_wide(&buffer, copied)
    }

    fn window_class(window: HWND) -> String {
        let mut buffer = [0u16; 256];
        let copied = unsafe { GetClassNameW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
        from_wide(&buffer, copied)
    }

    fn from_wide(buffer: &[u16], copied: i32) -> String {
        if copied <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buffer[..copied as usize]).trim().to_string()
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::{FocusChangeCallback, FocusVerdict};
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

    /// Longest property value we care to read, in 32 bit units.
    const MAX_PROPERTY_WORDS: u32 = 1024;

    pub fn spawn(callback: FocusChangeCallback, running: Arc<AtomicBool>) {
        if std::env::var_os("DISPLAY").is_none() {
            log_bus::log("[focus] focus monitoring needs an X display");
            running.store(false, Ordering::SeqCst);
            return;
        }

        std::thread::Builder::new()
            .name("focus-monitor".to_string())
            .spawn(move || {
                if let Err(error) = run(&callback, &running) {
                    log_bus::log(format!("[focus:error] {error}"));
                }
                running.store(false, Ordering::SeqCst);
                log_bus::log("[focus] monitoring thread finished");
            })
            .expect("focus monitor thread can be spawned");
    }

    fn run(callback: &FocusChangeCallback, running: &AtomicBool) -> Result<(), String> {
        let (connection, screen_index) = x11rb::connect(None).map_err(|error| error.to_string())?;
        let root = connection.setup().roots[screen_index].root;

        let active_window = intern(&connection, b"_NET_ACTIVE_WINDOW")?;
        let net_wm_name = intern(&connection, b"_NET_WM_NAME")?;
        let utf8_string = intern(&connection, b"UTF8_STRING")?;

        log_bus::log("[focus] monitoring thread started");
        let mut last_window = None;

        while running.load(Ordering::SeqCst) {
            if let Some(window) = focused_window(&connection, root, active_window)
                && last_window != Some(window)
            {
                last_window = Some(window);

                let title = title_of(&connection, window, net_wm_name, utf8_string);
                let class_name = class_of(&connection, window);
                let verdict = FocusVerdict::of(&title, &class_name);

                log_bus::log(format!("[focus] focus moved to: {title} ({class_name}) -> {verdict:?}"));
                callback(verdict);
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    fn intern(connection: &impl Connection, name: &[u8]) -> Result<u32, String> {
        connection
            .intern_atom(false, name)
            .map_err(|error| error.to_string())?
            .reply()
            .map(|reply| reply.atom)
            .map_err(|error| error.to_string())
    }

    /// Reads `_NET_ACTIVE_WINDOW` off the root window, which is how EWMH
    /// compliant window managers publish the focused window.
    fn focused_window(connection: &impl Connection, root: Window, active_window: u32) -> Option<Window> {
        let reply = connection
            .get_property(false, root, active_window, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        reply.value32()?.next().filter(|window| *window != 0)
    }

    /// Prefers the UTF-8 `_NET_WM_NAME`, falling back to the legacy `WM_NAME`.
    fn title_of(connection: &impl Connection, window: Window, net_wm_name: u32, utf8_string: u32) -> String {
        let utf8 = text_property(connection, window, net_wm_name, utf8_string);
        if !utf8.is_empty() {
            return utf8;
        }
        text_property(connection, window, AtomEnum::WM_NAME.into(), AtomEnum::STRING.into())
    }

    /// `WM_CLASS` holds an instance name and a class name, NUL separated; the
    /// class name is the one that identifies the application.
    fn class_of(connection: &impl Connection, window: Window) -> String {
        let raw = text_property(connection, window, AtomEnum::WM_CLASS.into(), AtomEnum::STRING.into());
        raw.split('\0').nth(1).unwrap_or(&raw).trim().to_string()
    }

    fn text_property(connection: &impl Connection, window: Window, property: u32, kind: u32) -> String {
        let Ok(cookie) = connection.get_property(false, window, property, kind, 0, MAX_PROPERTY_WORDS) else {
            return String::new();
        };
        let Ok(reply) = cookie.reply() else { return String::new() };

        String::from_utf8_lossy(&reply.value).trim_end_matches('\0').trim().to_string()
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    use super::FocusChangeCallback;
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Outside Windows and X11 there is no way to observe the focused window,
    /// so the monitor stays idle and leaves the hotkey untouched.
    pub fn spawn(_callback: FocusChangeCallback, running: Arc<AtomicBool>) {
        log_bus::log("[focus] focus monitoring is only available on Windows");
        running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editors_and_games_release_the_hotkey() {
        assert!(should_disable_hotkey("main.rs - Visual Studio Code", "Chrome_WidgetWin_1"));
        assert!(should_disable_hotkey("Untitled - Notepad", "Notepad"));
        assert!(should_disable_hotkey("Elden Ring", "UnrealWindow"));
    }

    #[test]
    fn browsers_keep_the_hotkey_despite_a_shared_window_class() {
        assert!(!should_disable_hotkey("Anthropic - Google Chrome", "Chrome_WidgetWin_1"));
        assert!(!should_disable_hotkey("Docs - Mozilla Firefox", "MozillaWindowClass"));
    }

    #[test]
    fn ordinary_windows_keep_the_hotkey() {
        assert!(!should_disable_hotkey("File Organizer", "FileOrganizerWindow"));
        assert!(!should_disable_hotkey("", ""));
    }
}
