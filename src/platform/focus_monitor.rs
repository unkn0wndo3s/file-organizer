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

/// Window classes used by browsers, which must keep the hotkey available.
const BROWSER_CLASSES: &[&str] = &[
    "Chrome_WidgetWin_1",
    "Chrome_WidgetWin_0",
    "MozillaWindowClass",
    "MozillaUIWindow",
    "IEFrame",
    "EdgeUiInputTopWndClass",
    "ApplicationFrameWindow",
    "Safari",
    "OperaWindow",
    "Vivaldi",
    "Brave",
    "TorBrowser",
    "Waterfox",
    "PaleMoon",
    "SeaMonkey",
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
pub type FocusChangeCallback = Arc<dyn Fn(FocusChange) + Send + Sync + 'static>;

/// Describes the window that just took focus.
#[derive(Clone, Debug)]
pub struct FocusChange {
    pub window_title: String,
    pub class_name: String,
    pub should_disable_hotkey: bool,
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
    use super::{should_disable_hotkey, FocusChange, FocusChangeCallback};
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

                        let window_title = window_text(current);
                        let class_name = window_class(current);
                        let should_disable = should_disable_hotkey(&window_title, &class_name);

                        log_bus::log(format!("[focus] focus moved to: {window_title} ({class_name})"));
                        if should_disable {
                            log_bus::log("[focus] focused application requires the hotkey to be released");
                        }

                        callback(FocusChange { window_title, class_name, should_disable_hotkey: should_disable });
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

#[cfg(not(windows))]
mod imp {
    use super::FocusChangeCallback;
    use crate::core::log_bus;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Focus tracking relies on the Win32 foreground window, so elsewhere the
    /// monitor simply stays idle and leaves the hotkey untouched.
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
