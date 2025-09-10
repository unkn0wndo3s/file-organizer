package fr.unkn0wndo3s.windows;

import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.platform.win32.WinUser;
import com.sun.jna.platform.win32.WinUser.MSG;

import fr.unkn0wndo3s.core.LogBus;

public class WindowsHotkeyService {

    public interface HotkeyCallback {
        void onToggle();
    }

    private final HotkeyCallback callback;
    private Thread loopThread;
    private volatile boolean running = false;

    private static final int HOTKEY_ID = 0xBEEF;
    private static final int MOD_CONTROL = 0x0002;
    private static final int MOD_ALT = 0x0001;
    private static final int MOD_WIN = 0x0008;
    private static final int VK_SPACE    = 0x20;
    private static final int VK_F12 = 0x7B;  // F12 comme alternative
    private static final int VK_F11 = 0x7A;  // F11 comme alternative
    private static final int VK_G = 0x47;    // G comme alternative

    public WindowsHotkeyService(HotkeyCallback callback) {
        this.callback = callback;
    }

    public void start() {
        if (running) {
            LogBus.log("[hotkey] Service déjà démarré");
            return;
        }
        running = true;
        LogBus.log("[hotkey] Démarrage du service de raccourcis...");

        loopThread = new Thread(() -> {
            try {
                System.out.println("[HOTKEY-DEBUG] Thread de raccourcis démarré");
                LogBus.log("[hotkey] Thread de raccourcis démarré");
                System.out.println("[HOTKEY-DEBUG] Tentative d'enregistrement du raccourci Ctrl+Space...");
                LogBus.log("[hotkey] Tentative d'enregistrement du raccourci Ctrl+Space...");
                boolean ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_CONTROL, VK_SPACE);
                System.out.println("[HOTKEY-DEBUG] Résultat RegisterHotKey Ctrl+Space: " + ok);
                if (!ok) { 
                    int errorCode = com.sun.jna.platform.win32.Kernel32.INSTANCE.GetLastError();
                    System.out.println("[HOTKEY-DEBUG] Erreur Ctrl+Space, code: " + errorCode);
                    LogBus.log("[hotkey:error] Impossible d'enregistrer le raccourci Ctrl+Space (code: " + errorCode + ")");
                    String errorMsg = getErrorMessage(errorCode);
                    System.out.println("[HOTKEY-DEBUG] Message d'erreur: " + errorMsg);
                    LogBus.log("[hotkey:error] Message d'erreur: " + errorMsg);
                    System.out.println("[HOTKEY-DEBUG] Tentative avec Alt+F12...");
                    LogBus.log("[hotkey] Tentative avec Alt+F12...");
                    ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_ALT, VK_F12);
                    System.out.println("[HOTKEY-DEBUG] Résultat RegisterHotKey Alt+F12: " + ok);
                    if (!ok) {
                        errorCode = com.sun.jna.platform.win32.Kernel32.INSTANCE.GetLastError();
                        System.out.println("[HOTKEY-DEBUG] Erreur Alt+F12, code: " + errorCode);
                        LogBus.log("[hotkey:error] Impossible d'enregistrer le raccourci Alt+F12 (code: " + errorCode + ")");
                        errorMsg = getErrorMessage(errorCode);
                        System.out.println("[HOTKEY-DEBUG] Message d'erreur: " + errorMsg);
                        LogBus.log("[hotkey:error] Message d'erreur: " + errorMsg);
                        System.out.println("[HOTKEY-DEBUG] Tentative avec Win+G...");
                        LogBus.log("[hotkey] Tentative avec Win+G...");
                        ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_WIN, VK_G);
                        System.out.println("[HOTKEY-DEBUG] Résultat RegisterHotKey Win+G: " + ok);
                        if (!ok) {
                            errorCode = com.sun.jna.platform.win32.Kernel32.INSTANCE.GetLastError();
                            System.out.println("[HOTKEY-DEBUG] Erreur Win+G, code: " + errorCode);
                            LogBus.log("[hotkey:error] Impossible d'enregistrer le raccourci Win+G (code: " + errorCode + ")");
                            errorMsg = getErrorMessage(errorCode);
                            System.out.println("[HOTKEY-DEBUG] Message d'erreur: " + errorMsg);
                            LogBus.log("[hotkey:error] Message d'erreur: " + errorMsg);
                            System.out.println("[HOTKEY-DEBUG] Tentative avec F11...");
                            LogBus.log("[hotkey] Tentative avec F11...");
                            ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, 0, VK_F11);
                            System.out.println("[HOTKEY-DEBUG] Résultat RegisterHotKey F11: " + ok);
                            if (!ok) {
                                errorCode = com.sun.jna.platform.win32.Kernel32.INSTANCE.GetLastError();
                                System.out.println("[HOTKEY-DEBUG] Erreur F11, code: " + errorCode);
                                LogBus.log("[hotkey:error] Impossible d'enregistrer le raccourci F11 (code: " + errorCode + ")");
                                errorMsg = getErrorMessage(errorCode);
                                System.out.println("[HOTKEY-DEBUG] Message d'erreur: " + errorMsg);
                                LogBus.log("[hotkey:error] Message d'erreur: " + errorMsg);
                                running = false; 
                                System.out.println("[HOTKEY-DEBUG] Arrêt du service - aucun raccourci disponible");
                                LogBus.log("[hotkey:error] Arrêt du service - aucun raccourci disponible");
                                return; 
                            }
                            System.out.println("[HOTKEY-DEBUG] F11 enregistré avec succès");
                            LogBus.log("[hotkey] Raccourci F11 enregistré avec succès");
                        } else {
                            System.out.println("[HOTKEY-DEBUG] Win+G enregistré avec succès");
                            LogBus.log("[hotkey] Raccourci Win+G enregistré avec succès");
                        }
                    } else {
                        System.out.println("[HOTKEY-DEBUG] Alt+F12 enregistré avec succès");
                        LogBus.log("[hotkey] Raccourci Alt+F12 enregistré avec succès");
                    }
                } else {
                    System.out.println("[HOTKEY-DEBUG] Ctrl+Space enregistré avec succès");
                    LogBus.log("[hotkey] Raccourci Ctrl+Space enregistré avec succès");
                }

                System.out.println("[HOTKEY-DEBUG] Démarrage de la boucle de messages...");
                LogBus.log("[hotkey] Démarrage de la boucle de messages...");
                MSG msg = new MSG();
                while (running) {
                    int result = User32.INSTANCE.GetMessage(msg, null, 0, 0);
                    if (result == 0) {
                        System.out.println("[HOTKEY-DEBUG] WM_QUIT reçu, arrêt de la boucle");
                        LogBus.log("[hotkey] WM_QUIT reçu, arrêt de la boucle");
                        break;      // WM_QUIT
                    }
                    if (result == -1) {
                        System.out.println("[HOTKEY-DEBUG] GetMessage a échoué");
                        LogBus.log("[hotkey:error] GetMessage a échoué");
                        break;     // erreur
                    }
                
                    if (msg.message == WinUser.WM_HOTKEY && msg.wParam.intValue() == HOTKEY_ID) {
                        System.out.println("[HOTKEY-DEBUG] Raccourci détecté!");
                        LogBus.log("[hotkey] Raccourci détecté!");
                        if (callback != null) {
                            System.out.println("[HOTKEY-DEBUG] Exécution du callback...");
                            LogBus.log("[hotkey] Exécution du callback...");
                            callback.onToggle();
                        }
                    } else {
                        User32.INSTANCE.TranslateMessage(msg);
                        User32.INSTANCE.DispatchMessage(msg);
                    }
                }
                System.out.println("[HOTKEY-DEBUG] Boucle de messages terminée");
                LogBus.log("[hotkey] Boucle de messages terminée");
            } catch (Exception e) {
                System.out.println("[HOTKEY-DEBUG] Exception dans le thread: " + e.getMessage());
                LogBus.log("[hotkey:error] " + e.getMessage());
                e.printStackTrace();
            } finally {
                try {
                    System.out.println("[HOTKEY-DEBUG] Désenregistrement du raccourci...");
                    User32.INSTANCE.UnregisterHotKey(null, HOTKEY_ID);
                    System.out.println("[HOTKEY-DEBUG] Raccourci désenregistré");
                    LogBus.log("[hotkey] Raccourci désenregistré");
                } catch (Exception e) {
                    System.out.println("[HOTKEY-DEBUG] Erreur lors du désenregistrement: " + e.getMessage());
                    LogBus.log("[hotkey:error] Erreur lors du désenregistrement: " + e.getMessage());
                }
            }
        }, "HotkeyLoop");
        loopThread.setDaemon(true);
        loopThread.start();
    }

    public void stop() {
        LogBus.log("[hotkey] Arrêt du service de raccourcis...");
        running = false;
        try { 
            User32.INSTANCE.PostQuitMessage(0); 
            LogBus.log("[hotkey] PostQuitMessage envoyé");
        } catch (Throwable e) {
            LogBus.log("[hotkey:error] Erreur lors de PostQuitMessage: " + e.getMessage());
        }
    }

    private String getErrorMessage(int errorCode) {
        switch (errorCode) {
            case 1409: return "Hotkey already registered (ERROR_HOTKEY_ALREADY_REGISTERED)";
            case 5: return "Access denied (ERROR_ACCESS_DENIED)";
            case 87: return "Invalid parameter (ERROR_INVALID_PARAMETER)";
            case 8: return "Not enough memory (ERROR_NOT_ENOUGH_MEMORY)";
            case 1408: return "Invalid window handle (ERROR_INVALID_WINDOW_HANDLE)";
            default: return "Unknown error code: " + errorCode;
        }
    }
}
