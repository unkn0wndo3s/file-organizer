package fr.unkn0wndo3s.windows;

import com.sun.jna.platform.win32.Kernel32;
import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.platform.win32.WinUser;
import com.sun.jna.platform.win32.WinUser.MSG;

import fr.unkn0wndo3s.core.LogBus;

public class WindowsHotkeyService {

    public interface HotkeyCallback {
        void onToggle();
    }

    public interface HotkeyRegistrationCallback {
        void onHotkeyRegistered(String hotkeyName);
        void onHotkeyRegistrationFailed(String errorMessage);
    }

    private final HotkeyCallback callback;
    private final HotkeyRegistrationCallback registrationCallback;
    private Thread loopThread;
    private volatile boolean running = false;
    private WindowsFocusMonitor focusMonitor;
    private volatile boolean hotkeyRegistered = false;
    private volatile boolean hotkeyActive = true; // État secondaire : true = active, false = ignorée

    private static final int HOTKEY_ID = 0xBEEF;
    private static final int MOD_CONTROL = 0x0002;
    private static final int VK_SPACE    = 0x20;

    public WindowsHotkeyService(HotkeyCallback callback) {
        this.callback = callback;
        this.registrationCallback = null;
    }

    public WindowsHotkeyService(HotkeyCallback callback, HotkeyRegistrationCallback registrationCallback) {
        this.callback = callback;
        this.registrationCallback = registrationCallback;
    }

    public void start() {
        if (running) {
            LogBus.log("[hotkey] Service déjà démarré");
            return;
        }
        running = true;
        LogBus.log("[hotkey] Démarrage du service de raccourcis...");
        
        // Démarrer le monitoring de focus
        focusMonitor = new WindowsFocusMonitor((title, className, shouldDisableHotkey) -> {
            if (shouldDisableHotkey) {
                deactivateHotkey();
            } else {
                activateHotkey();
            }
        });
        focusMonitor.start();

        loopThread = new Thread(() -> {
            try {
                System.out.println("[HOTKEY-DEBUG] Thread de raccourcis démarré");
                LogBus.log("[hotkey] Thread de raccourcis démarré");
                
                // Enregistrer la hotkey initiale
                registerHotkey();

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
                        if (hotkeyRegistered) {
                            System.out.println("[HOTKEY-DEBUG] Raccourci détecté!");
                            LogBus.log("[hotkey] Raccourci détecté!");
                            if (callback != null) {
                                System.out.println("[HOTKEY-DEBUG] Exécution du callback...");
                                LogBus.log("[hotkey] Exécution du callback...");
                                callback.onToggle();
                            }
                        } else {
                            System.out.println("[HOTKEY-DEBUG] Raccourci ignoré (hotkey non enregistrée)");
                            LogBus.log("[hotkey] Raccourci ignoré (hotkey non enregistrée)");
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
        
        // Arrêter le monitoring de focus
        if (focusMonitor != null) {
            focusMonitor.stop();
        }
        
        try { 
            User32.INSTANCE.PostQuitMessage(0); 
            LogBus.log("[hotkey] PostQuitMessage envoyé");
        } catch (Throwable e) {
            LogBus.log("[hotkey:error] Erreur lors de PostQuitMessage: " + e.getMessage());
        }
    }

    private void registerHotkey() {
        System.out.println("[HOTKEY-DEBUG] Enregistrement de la hotkey...");
        LogBus.log("[hotkey] Enregistrement de la hotkey...");
        
        try {
            boolean ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_CONTROL, VK_SPACE);
            
            if (ok) {
                hotkeyRegistered = true;
                hotkeyActive = true;
                System.out.println("[HOTKEY-DEBUG] Hotkey enregistrée avec succès");
                LogBus.log("[hotkey] Hotkey enregistrée avec succès");
                if (registrationCallback != null) {
                    registrationCallback.onHotkeyRegistered("Ctrl+Space");
                }
            } else {
                int errorCode = Kernel32.INSTANCE.GetLastError();
                System.out.println("[HOTKEY-DEBUG] Erreur enregistrement, code: " + errorCode);
                String errorMsg = getErrorMessage(errorCode);
                System.out.println("[HOTKEY-DEBUG] Message d'erreur: " + errorMsg);
                
                // Si l'erreur est "already registered", on considère que c'est OK
                if (errorCode == 1409) { // ERROR_HOTKEY_ALREADY_REGISTERED
                    hotkeyRegistered = true; // On considère qu'elle est enregistrée
                    hotkeyActive = true;
                    System.out.println("[HOTKEY-DEBUG] Hotkey déjà enregistrée par ailleurs, considéré comme OK");
                    LogBus.log("[hotkey] Hotkey déjà enregistrée par ailleurs, considéré comme OK");
                    if (registrationCallback != null) {
                        registrationCallback.onHotkeyRegistered("Ctrl+Space");
                    }
                } else {
                    hotkeyRegistered = false;
                    hotkeyActive = false;
                    LogBus.log("[hotkey:error] Impossible d'enregistrer la hotkey (code: " + errorCode + ")");
                    LogBus.log("[hotkey:error] Message d'erreur: " + errorMsg);
                    
                    if (registrationCallback != null) {
                        registrationCallback.onHotkeyRegistrationFailed("Ctrl+Space: " + errorMsg);
                    }
                }
            }
        } catch (Exception e) {
            hotkeyRegistered = false;
            hotkeyActive = false;
            System.out.println("[HOTKEY-DEBUG] Exception lors de l'enregistrement: " + e.getMessage());
            LogBus.log("[hotkey:error] Exception lors de l'enregistrement: " + e.getMessage());
            
            if (registrationCallback != null) {
                registrationCallback.onHotkeyRegistrationFailed("Ctrl+Space: " + e.getMessage());
            }
        }
    }
    
    private void activateHotkey() {
        if (!hotkeyRegistered) {
            System.out.println("[HOTKEY-DEBUG] Hotkey non enregistrée, enregistrement...");
            registerHotkey();
            return;
        }
        
        hotkeyActive = true;
        System.out.println("[HOTKEY-DEBUG] Hotkey activée");
        LogBus.log("[hotkey] Hotkey activée");
    }
    
    private void deactivateHotkey() {
        if (!hotkeyRegistered) {
            System.out.println("[HOTKEY-DEBUG] Hotkey non enregistrée, ignoré");
            LogBus.log("[hotkey] Hotkey non enregistrée, ignoré");
            return;
        }
        
        // Désenregistrer vraiment la hotkey de Windows pour libérer Ctrl+Space
        System.out.println("[HOTKEY-DEBUG] Désenregistrement de la hotkey pour libérer Ctrl+Space...");
        LogBus.log("[hotkey] Désenregistrement de la hotkey pour libérer Ctrl+Space...");
        
        try {
            boolean ok = User32.INSTANCE.UnregisterHotKey(null, HOTKEY_ID);
            if (ok) {
                hotkeyRegistered = false;
                hotkeyActive = false;
                System.out.println("[HOTKEY-DEBUG] Hotkey désenregistrée avec succès - Ctrl+Space libéré");
                LogBus.log("[hotkey] Hotkey désenregistrée avec succès - Ctrl+Space libéré");
            } else {
                int errorCode = Kernel32.INSTANCE.GetLastError();
                if (errorCode == 1419) { // ERROR_HOTKEY_NOT_REGISTERED
                    hotkeyRegistered = false;
                    hotkeyActive = false;
                    System.out.println("[HOTKEY-DEBUG] Hotkey n'était pas enregistrée - Ctrl+Space déjà libéré");
                    LogBus.log("[hotkey] Hotkey n'était pas enregistrée - Ctrl+Space déjà libéré");
                } else {
                    hotkeyActive = false;
                    System.out.println("[HOTKEY-DEBUG] Erreur désenregistrement (code: " + errorCode + "), état forcé à inactif");
                    LogBus.log("[hotkey:warning] Erreur désenregistrement (code: " + errorCode + "), état forcé à inactif");
                }
            }
        } catch (Exception e) {
            hotkeyActive = false;
            System.out.println("[HOTKEY-DEBUG] Exception lors du désenregistrement: " + e.getMessage());
            LogBus.log("[hotkey:warning] Exception lors du désenregistrement: " + e.getMessage());
        }
    }
    


    private String getErrorMessage(int errorCode) {
        switch (errorCode) {
            case 1409: return "Hotkey already registered (ERROR_HOTKEY_ALREADY_REGISTERED)";
            case 1419: return "Hotkey not registered (ERROR_HOTKEY_NOT_REGISTERED)";
            case 5: return "Access denied (ERROR_ACCESS_DENIED)";
            case 87: return "Invalid parameter (ERROR_INVALID_PARAMETER)";
            case 8: return "Not enough memory (ERROR_NOT_ENOUGH_MEMORY)";
            case 1408: return "Invalid window handle (ERROR_INVALID_WINDOW_HANDLE)";
            default: return "Unknown error code: " + errorCode;
        }
    }
}