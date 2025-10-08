package fr.unkn0wndo3s.windows;

import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.ptr.IntByReference;

import fr.unkn0wndo3s.core.LogBus;

public class WindowsFocusMonitor {
    
    public interface FocusChangeCallback {
        void onFocusChange(String windowTitle, String className, boolean shouldDisableHotkey);
    }
    
    private final FocusChangeCallback callback;
    private Thread monitorThread;
    private volatile boolean running = false;
    private HWND lastFocusedWindow = null;
    
    // Common window classes for text editors
    private static final String[] TEXT_EDITOR_CLASSES = {
        "Notepad",           // Notepad
        "Vim",               // Vim
        "VimGtk",            // Vim GTK
        "Code",              // VS Code
        "Chrome_WidgetWin_1", // VS Code, Cursor (Electron)
        "Chrome_WidgetWin_0", // VS Code, Cursor (Electron variant)
        "SunAwtFrame",       // Java applications (editors)
        "SWT_Window0",       // Eclipse, IntelliJ
        "ConsoleWindowClass", // Windows Console
        "Edit",              // Simple edit controls
        "RichEdit20W",       // Rich edit controls
        "RICHEDIT50W",       // Modern rich edit controls
        "Scintilla",         // Scintilla (used by Notepad++, etc.)
        "wxWindowClassNR",   // wxWidgets applications
        "TkTopLevel",        // Tkinter applications (Python)
        "Qt5QWindowIcon",    // Qt5 applications
        "Qt6QWindowIcon",    // Qt6 applications
        "CefBrowserWindow",  // CEF-based applications (editors)
        "ElectronMainWindow", // Electron applications
        "Chrome_RenderWidgetHostHWND", // Electron/CEF applications
        "Chrome_WidgetWin_2" // Electron applications (variant)
    };
    
    // Common window classes for games
    private static final String[] GAME_CLASSES = {
        "UnityWndClass",     // Unity games
        "UnrealWindow",      // Unreal Engine games
        "Valve001",          // Source games (Half-Life, CS, etc.)
        "TankWindowClass",   // Games using Tank Engine
        "CryEngine",         // CryEngine games
        "Frostbite",         // Frostbite games
        "D3DWindow",         // DirectX games
        "OpenGLWindow",      // OpenGL games
        "SDL_app",           // SDL games
        "AllegroWindow",     // Allegro games
        "SFML_Window",       // SFML games
        "GLFW3",             // GLFW games
        "SDL_Window",        // SDL2 games
        "GameWindow",        // Generic games
        "MainWindow",        // Games with main window
        "RenderWindow",      // Render windows
        "DirectXWindow",     // DirectX windows
        "VulkanWindow",      // Vulkan windows
        "MetalWindow",       // Metal windows (macOS)
        "GameEngine"         // Game engines
    };
    
    // Window classes for browsers (should NOT disable hotkey)
    private static final String[] BROWSER_CLASSES = {
        "Chrome_WidgetWin_1", // Chrome
        "Chrome_WidgetWin_0", // Chrome (variant)
        "MozillaWindowClass", // Firefox
        "MozillaUIWindow",    // Firefox (variant)
        "IEFrame",           // Internet Explorer
        "EdgeUiInputTopWndClass", // Edge
        "ApplicationFrameWindow", // Edge (variant)
        "Safari",            // Safari
        "OperaWindow",       // Opera
        "Vivaldi",           // Vivaldi
        "Brave",             // Brave
        "TorBrowser",        // Tor Browser
        "Waterfox",          // Waterfox
        "PaleMoon",          // Pale Moon
        "SeaMonkey"          // SeaMonkey
    };
    
    // Keywords in window titles to detect editors
    private static final String[] TEXT_EDITOR_KEYWORDS = {
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
        "fade in", "highland", "writerduet", "trelby", "kit scenarist", "kitscenarist"
    };
    
    // Keywords in window titles to detect games
    private static final String[] GAME_KEYWORDS = {
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
        "nier", "bayonetta", "devil may cry", "darksiders", "darksiders",
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
        "game", "gaming", "play", "playing", "launcher", "launcher.exe"
    };
    
    // Keywords in window titles to detect browsers (should NOT disable hotkey)
    private static final String[] BROWSER_KEYWORDS = {
        "chrome", "firefox", "edge", "safari", "opera", "vivaldi", "brave",
        "tor browser", "waterfox", "pale moon", "seamonkey", "internet explorer",
        "microsoft edge", "google chrome", "mozilla firefox", "mozilla firefox",
        "browser", "web browser", "navigateur", "navigateur web"
    };
    
    public WindowsFocusMonitor(FocusChangeCallback callback) {
        this.callback = callback;
    }
    
    public void start() {
        if (running) {
            LogBus.log("[focus] Monitor already started");
            return;
        }
        
        running = true;
        LogBus.log("[focus] Starting focus monitoring...");
        
        monitorThread = new Thread(() -> {
            try {
                System.out.println("[FOCUS-DEBUG] Monitoring thread started");
                LogBus.log("[focus] Monitoring thread started");
                
                while (running) {
                    try {
                        HWND currentWindow = User32.INSTANCE.GetForegroundWindow();
                        
                        if (currentWindow != null && !currentWindow.equals(lastFocusedWindow)) {
                            lastFocusedWindow = currentWindow;
                            
                            String windowTitle = getWindowTitle(currentWindow);
                            String className = getWindowClassName(currentWindow);
                            
                            System.out.println("[FOCUS-DEBUG] Focus change to: " + windowTitle);
                            System.out.println("[FOCUS-DEBUG] Window class: " + className);
                            LogBus.log("[focus] Focus change to: " + windowTitle);
                            LogBus.log("[focus] Window class: " + className);
                            
                            boolean shouldDisable = shouldDisableHotkey(windowTitle, className);
                            
                            if (shouldDisable) {
                                System.out.println("[FOCUS-DEBUG] Focus on application requiring hotkey deactivation");
                                LogBus.log("[focus] Focus on application requiring hotkey deactivation");
                            } else {
                                System.out.println("[FOCUS-DEBUG] Focus on normal application");
                                LogBus.log("[focus] Focus on normal application");
                            }
                            
                            if (callback != null) {
                                callback.onFocusChange(windowTitle, className, shouldDisable);
                            }
                        }
                        
                        Thread.sleep(100); // Check every 100ms
                        
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        break;
                    } catch (Exception e) {
                        System.out.println("[FOCUS-DEBUG] Error in monitoring: " + e.getMessage());
                        LogBus.log("[focus:error] " + e.getMessage());
                        Thread.sleep(1000); // Wait 1s in case of error
                    }
                }
                
                System.out.println("[FOCUS-DEBUG] Monitoring thread finished");
                LogBus.log("[focus] Monitoring thread finished");
                
            } catch (Exception e) {
                System.out.println("[FOCUS-DEBUG] Exception in thread: " + e.getMessage());
                LogBus.log("[focus:error] " + e.getMessage());
                e.printStackTrace();
            }
        }, "FocusMonitor");
        
        monitorThread.setDaemon(true);
        monitorThread.start();
    }
    
    public void stop() {
        LogBus.log("[focus] Stopping focus monitoring...");
        running = false;
        if (monitorThread != null) {
            monitorThread.interrupt();
        }
    }
    
    private String getWindowTitle(HWND hwnd) {
        try {
            IntByReference length = new IntByReference();
            User32.INSTANCE.GetWindowTextLength(hwnd);
            int len = User32.INSTANCE.GetWindowTextLength(hwnd);
            if (len == 0) return "";
            
            char[] buffer = new char[len + 1];
            User32.INSTANCE.GetWindowText(hwnd, buffer, len + 1);
            return new String(buffer).trim();
        } catch (Exception e) {
            return "";
        }
    }
    
    private String getWindowClassName(HWND hwnd) {
        try {
            char[] buffer = new char[256];
            User32.INSTANCE.GetClassName(hwnd, buffer, buffer.length);
            return new String(buffer).trim();
        } catch (Exception e) {
            return "";
        }
    }
    
    private boolean shouldDisableHotkey(String title, String className) {
        if (title == null) title = "";
        if (className == null) className = "";
        
        String titleLower = title.toLowerCase();
        String classLower = className.toLowerCase();
        
        // First check if it's a specific editor by title (high priority)
        for (String keyword : TEXT_EDITOR_KEYWORDS) {
            if (titleLower.contains(keyword)) {
                return true;
            }
        }
        
        // Check if it's a game by title (high priority)
        for (String keyword : GAME_KEYWORDS) {
            if (titleLower.contains(keyword)) {
                return true;
            }
        }
        
        // Check window classes for editors
        for (String editorClass : TEXT_EDITOR_CLASSES) {
            if (classLower.contains(editorClass.toLowerCase())) {
                // But exclude browsers even if they have the same class
                boolean isBrowser = false;
                for (String browserKeyword : BROWSER_KEYWORDS) {
                    if (titleLower.contains(browserKeyword)) {
                        isBrowser = true;
                        break;
                    }
                }
                if (!isBrowser) {
                    return true;
                }
            }
        }
        
        // Check window classes for games
        for (String gameClass : GAME_CLASSES) {
            if (classLower.contains(gameClass.toLowerCase())) {
                return true;
            }
        }
        
        // Check if it's a browser (should NOT disable)
        for (String browserClass : BROWSER_CLASSES) {
            if (classLower.contains(browserClass.toLowerCase())) {
                return false; // Don't disable for browsers
            }
        }
        
        for (String browserKeyword : BROWSER_KEYWORDS) {
            if (titleLower.contains(browserKeyword)) {
                return false; // Don't disable for browsers
            }
        }
        
        return false;
    }
}
