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
    private volatile boolean hotkeyActive = true; // Secondary state: true = active, false = ignored

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
            LogBus.log("[hotkey] Service already started");
            return;
        }
        running = true;
        LogBus.log("[hotkey] Starting hotkey service...");
        
        // Start focus monitoring
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
                System.out.println("[HOTKEY-DEBUG] Hotkey thread started");
                LogBus.log("[hotkey] Hotkey thread started");
                
                // Register initial hotkey
                registerHotkey();

                System.out.println("[HOTKEY-DEBUG] Starting message loop...");
                LogBus.log("[hotkey] Starting message loop...");
                MSG msg = new MSG();
                while (running) {
                    int result = User32.INSTANCE.GetMessage(msg, null, 0, 0);
                    if (result == 0) {
                        System.out.println("[HOTKEY-DEBUG] WM_QUIT received, stopping loop");
                        LogBus.log("[hotkey] WM_QUIT received, stopping loop");
                        break;      // WM_QUIT
                    }
                    if (result == -1) {
                        System.out.println("[HOTKEY-DEBUG] GetMessage failed");
                        LogBus.log("[hotkey:error] GetMessage failed");
                        break;     // erreur
                    }
                
                    if (msg.message == WinUser.WM_HOTKEY && msg.wParam.intValue() == HOTKEY_ID) {
                        if (hotkeyRegistered) {
                        System.out.println("[HOTKEY-DEBUG] Hotkey detected!");
                        LogBus.log("[hotkey] Hotkey detected!");
                            if (callback != null) {
                                System.out.println("[HOTKEY-DEBUG] Executing callback...");
                                LogBus.log("[hotkey] Executing callback...");
                                callback.onToggle();
                            }
                        } else {
                            System.out.println("[HOTKEY-DEBUG] Hotkey ignored (hotkey not registered)");
                            LogBus.log("[hotkey] Hotkey ignored (hotkey not registered)");
                        }
                    } else {
                        User32.INSTANCE.TranslateMessage(msg);
                        User32.INSTANCE.DispatchMessage(msg);
                    }
                }
                System.out.println("[HOTKEY-DEBUG] Message loop finished");
                LogBus.log("[hotkey] Message loop finished");
            } catch (Exception e) {
                System.out.println("[HOTKEY-DEBUG] Exception in thread: " + e.getMessage());
                LogBus.log("[hotkey:error] " + e.getMessage());
                e.printStackTrace();
            } finally {
                try {
                    System.out.println("[HOTKEY-DEBUG] Unregistering hotkey...");
                    User32.INSTANCE.UnregisterHotKey(null, HOTKEY_ID);
                    System.out.println("[HOTKEY-DEBUG] Hotkey unregistered");
                    LogBus.log("[hotkey] Hotkey unregistered");
                } catch (Exception e) {
                    System.out.println("[HOTKEY-DEBUG] Error during unregistration: " + e.getMessage());
                    LogBus.log("[hotkey:error] Error during unregistration: " + e.getMessage());
                }
            }
        }, "HotkeyLoop");
        loopThread.setDaemon(true);
        loopThread.start();
    }

    public void stop() {
        LogBus.log("[hotkey] Stopping hotkey service...");
        running = false;
        
        // Stop focus monitoring
        if (focusMonitor != null) {
            focusMonitor.stop();
        }
        
        try { 
            User32.INSTANCE.PostQuitMessage(0); 
            LogBus.log("[hotkey] PostQuitMessage sent");
        } catch (Throwable e) {
            LogBus.log("[hotkey:error] Error during PostQuitMessage: " + e.getMessage());
        }
    }

    private void registerHotkey() {
        System.out.println("[HOTKEY-DEBUG] Registering hotkey...");
        LogBus.log("[hotkey] Registering hotkey...");
        
        try {
            boolean ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_CONTROL, VK_SPACE);
            
            if (ok) {
                hotkeyRegistered = true;
                hotkeyActive = true;
                System.out.println("[HOTKEY-DEBUG] Hotkey registered successfully");
                LogBus.log("[hotkey] Hotkey registered successfully");
                if (registrationCallback != null) {
                    registrationCallback.onHotkeyRegistered("Ctrl+Space");
                }
            } else {
                int errorCode = Kernel32.INSTANCE.GetLastError();
                System.out.println("[HOTKEY-DEBUG] Registration error, code: " + errorCode);
                String errorMsg = getErrorMessage(errorCode);
                System.out.println("[HOTKEY-DEBUG] Error message: " + errorMsg);
                
                // If error is "already registered", consider it OK
                if (errorCode == 1409) { // ERROR_HOTKEY_ALREADY_REGISTERED
                    hotkeyRegistered = true; // Consider it registered
                    hotkeyActive = true;
                    System.out.println("[HOTKEY-DEBUG] Hotkey already registered elsewhere, considered OK");
                    LogBus.log("[hotkey] Hotkey already registered elsewhere, considered OK");
                    if (registrationCallback != null) {
                        registrationCallback.onHotkeyRegistered("Ctrl+Space");
                    }
                } else {
                    hotkeyRegistered = false;
                    hotkeyActive = false;
                    LogBus.log("[hotkey:error] Unable to register hotkey (code: " + errorCode + ")");
                    LogBus.log("[hotkey:error] Error message: " + errorMsg);
                    
                    if (registrationCallback != null) {
                        registrationCallback.onHotkeyRegistrationFailed("Ctrl+Space: " + errorMsg);
                    }
                }
            }
        } catch (Exception e) {
            hotkeyRegistered = false;
            hotkeyActive = false;
            System.out.println("[HOTKEY-DEBUG] Exception during registration: " + e.getMessage());
            LogBus.log("[hotkey:error] Exception during registration: " + e.getMessage());
            
            if (registrationCallback != null) {
                registrationCallback.onHotkeyRegistrationFailed("Ctrl+Space: " + e.getMessage());
            }
        }
    }
    
    private void activateHotkey() {
        if (!hotkeyRegistered) {
            System.out.println("[HOTKEY-DEBUG] Hotkey not registered, registering...");
            registerHotkey();
            return;
        }
        
        hotkeyActive = true;
        System.out.println("[HOTKEY-DEBUG] Hotkey activated");
        LogBus.log("[hotkey] Hotkey activated");
    }
    
    private void deactivateHotkey() {
        if (!hotkeyRegistered) {
            System.out.println("[HOTKEY-DEBUG] Hotkey not registered, ignored");
            LogBus.log("[hotkey] Hotkey not registered, ignored");
            return;
        }
        
        // Really unregister the hotkey from Windows to free Ctrl+Space
        System.out.println("[HOTKEY-DEBUG] Unregistering hotkey to free Ctrl+Space...");
        LogBus.log("[hotkey] Unregistering hotkey to free Ctrl+Space...");
        
        try {
            boolean ok = User32.INSTANCE.UnregisterHotKey(null, HOTKEY_ID);
            if (ok) {
                hotkeyRegistered = false;
                hotkeyActive = false;
                System.out.println("[HOTKEY-DEBUG] Hotkey unregistered successfully - Ctrl+Space freed");
                LogBus.log("[hotkey] Hotkey unregistered successfully - Ctrl+Space freed");
            } else {
                int errorCode = Kernel32.INSTANCE.GetLastError();
                if (errorCode == 1419) { // ERROR_HOTKEY_NOT_REGISTERED
                    hotkeyRegistered = false;
                    hotkeyActive = false;
                    System.out.println("[HOTKEY-DEBUG] Hotkey was not registered - Ctrl+Space already freed");
                    LogBus.log("[hotkey] Hotkey was not registered - Ctrl+Space already freed");
                } else {
                    hotkeyActive = false;
                    System.out.println("[HOTKEY-DEBUG] Unregistration error (code: " + errorCode + "), state forced to inactive");
                    LogBus.log("[hotkey:warning] Unregistration error (code: " + errorCode + "), state forced to inactive");
                }
            }
        } catch (Exception e) {
            hotkeyActive = false;
            System.out.println("[HOTKEY-DEBUG] Exception during unregistration: " + e.getMessage());
            LogBus.log("[hotkey:warning] Exception during unregistration: " + e.getMessage());
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