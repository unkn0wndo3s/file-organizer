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
    
    // Classes de fenêtre communes pour les éditeurs de texte
    private static final String[] TEXT_EDITOR_CLASSES = {
        "Notepad",           // Notepad
        "Vim",               // Vim
        "VimGtk",            // Vim GTK
        "Code",              // VS Code
        "Chrome_WidgetWin_1", // VS Code, Cursor (Electron)
        "Chrome_WidgetWin_0", // VS Code, Cursor (Electron variante)
        "SunAwtFrame",       // Applications Java (éditeurs)
        "SWT_Window0",       // Eclipse, IntelliJ
        "ConsoleWindowClass", // Console Windows
        "Edit",              // Contrôles d'édition simples
        "RichEdit20W",       // Contrôles d'édition riches
        "RICHEDIT50W",       // Contrôles d'édition riches modernes
        "Scintilla",         // Scintilla (utilisé par Notepad++, etc.)
        "wxWindowClassNR",   // Applications wxWidgets
        "TkTopLevel",        // Applications Tkinter (Python)
        "Qt5QWindowIcon",    // Applications Qt5
        "Qt6QWindowIcon",    // Applications Qt6
        "CefBrowserWindow",  // Applications basées sur CEF (éditeurs)
        "ElectronMainWindow", // Applications Electron
        "Chrome_RenderWidgetHostHWND", // Applications Electron/CEF
        "Chrome_WidgetWin_2" // Applications Electron (variante)
    };
    
    // Classes de fenêtre communes pour les jeux
    private static final String[] GAME_CLASSES = {
        "UnityWndClass",     // Jeux Unity
        "UnrealWindow",      // Jeux Unreal Engine
        "Valve001",          // Jeux Source (Half-Life, CS, etc.)
        "TankWindowClass",   // Jeux utilisant Tank Engine
        "CryEngine",         // Jeux CryEngine
        "Frostbite",         // Jeux Frostbite
        "D3DWindow",         // Jeux DirectX
        "OpenGLWindow",      // Jeux OpenGL
        "SDL_app",           // Jeux SDL
        "AllegroWindow",     // Jeux Allegro
        "SFML_Window",       // Jeux SFML
        "GLFW3",             // Jeux GLFW
        "SDL_Window",        // Jeux SDL2
        "GameWindow",        // Jeux génériques
        "MainWindow",        // Jeux avec fenêtre principale
        "RenderWindow",      // Fenêtres de rendu
        "DirectXWindow",     // Fenêtres DirectX
        "VulkanWindow",      // Fenêtres Vulkan
        "MetalWindow",       // Fenêtres Metal (macOS)
        "GameEngine"         // Moteurs de jeu
    };
    
    // Classes de fenêtre pour les navigateurs (à NE PAS désactiver)
    private static final String[] BROWSER_CLASSES = {
        "Chrome_WidgetWin_1", // Chrome
        "Chrome_WidgetWin_0", // Chrome (variante)
        "MozillaWindowClass", // Firefox
        "MozillaUIWindow",    // Firefox (variante)
        "IEFrame",           // Internet Explorer
        "EdgeUiInputTopWndClass", // Edge
        "ApplicationFrameWindow", // Edge (variante)
        "Safari",            // Safari
        "OperaWindow",       // Opera
        "Vivaldi",           // Vivaldi
        "Brave",             // Brave
        "TorBrowser",        // Tor Browser
        "Waterfox",          // Waterfox
        "PaleMoon",          // Pale Moon
        "SeaMonkey"          // SeaMonkey
    };
    
    // Mots-clés dans les titres de fenêtre pour détecter les éditeurs
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
    
    // Mots-clés dans les titres de fenêtre pour détecter les jeux
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
    
    // Mots-clés dans les titres de fenêtre pour détecter les navigateurs (à NE PAS désactiver)
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
            LogBus.log("[focus] Monitor déjà démarré");
            return;
        }
        
        running = true;
        LogBus.log("[focus] Démarrage du monitoring de focus...");
        
        monitorThread = new Thread(() -> {
            try {
                System.out.println("[FOCUS-DEBUG] Thread de monitoring démarré");
                LogBus.log("[focus] Thread de monitoring démarré");
                
                while (running) {
                    try {
                        HWND currentWindow = User32.INSTANCE.GetForegroundWindow();
                        
                        if (currentWindow != null && !currentWindow.equals(lastFocusedWindow)) {
                            lastFocusedWindow = currentWindow;
                            
                            String windowTitle = getWindowTitle(currentWindow);
                            String className = getWindowClassName(currentWindow);
                            
                            System.out.println("[FOCUS-DEBUG] Changement de focus vers: " + windowTitle);
                            System.out.println("[FOCUS-DEBUG] Classe de fenêtre: " + className);
                            LogBus.log("[focus] Changement de focus vers: " + windowTitle);
                            LogBus.log("[focus] Classe de fenêtre: " + className);
                            
                            boolean shouldDisable = shouldDisableHotkey(windowTitle, className);
                            
                            if (shouldDisable) {
                                System.out.println("[FOCUS-DEBUG] Focus sur une application nécessitant la désactivation de la hotkey");
                                LogBus.log("[focus] Focus sur une application nécessitant la désactivation de la hotkey");
                            } else {
                                System.out.println("[FOCUS-DEBUG] Focus sur une application normale");
                                LogBus.log("[focus] Focus sur une application normale");
                            }
                            
                            if (callback != null) {
                                callback.onFocusChange(windowTitle, className, shouldDisable);
                            }
                        }
                        
                        Thread.sleep(100); // Vérifier toutes les 100ms
                        
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        break;
                    } catch (Exception e) {
                        System.out.println("[FOCUS-DEBUG] Erreur dans le monitoring: " + e.getMessage());
                        LogBus.log("[focus:error] " + e.getMessage());
                        Thread.sleep(1000); // Attendre 1s en cas d'erreur
                    }
                }
                
                System.out.println("[FOCUS-DEBUG] Thread de monitoring terminé");
                LogBus.log("[focus] Thread de monitoring terminé");
                
            } catch (Exception e) {
                System.out.println("[FOCUS-DEBUG] Exception dans le thread: " + e.getMessage());
                LogBus.log("[focus:error] " + e.getMessage());
                e.printStackTrace();
            }
        }, "FocusMonitor");
        
        monitorThread.setDaemon(true);
        monitorThread.start();
    }
    
    public void stop() {
        LogBus.log("[focus] Arrêt du monitoring de focus...");
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
        
        // D'abord vérifier si c'est un éditeur spécifique par titre (priorité haute)
        for (String keyword : TEXT_EDITOR_KEYWORDS) {
            if (titleLower.contains(keyword)) {
                return true;
            }
        }
        
        // Vérifier si c'est un jeu par titre (priorité haute)
        for (String keyword : GAME_KEYWORDS) {
            if (titleLower.contains(keyword)) {
                return true;
            }
        }
        
        // Vérifier les classes de fenêtre pour les éditeurs
        for (String editorClass : TEXT_EDITOR_CLASSES) {
            if (classLower.contains(editorClass.toLowerCase())) {
                // Mais exclure les navigateurs même s'ils ont la même classe
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
        
        // Vérifier les classes de fenêtre pour les jeux
        for (String gameClass : GAME_CLASSES) {
            if (classLower.contains(gameClass.toLowerCase())) {
                return true;
            }
        }
        
        // Vérifier si c'est un navigateur (à NE PAS désactiver)
        for (String browserClass : BROWSER_CLASSES) {
            if (classLower.contains(browserClass.toLowerCase())) {
                return false; // Ne pas désactiver pour les navigateurs
            }
        }
        
        for (String browserKeyword : BROWSER_KEYWORDS) {
            if (titleLower.contains(browserKeyword)) {
                return false; // Ne pas désactiver pour les navigateurs
            }
        }
        
        return false;
    }
}
