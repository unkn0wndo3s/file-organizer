package fr.unkn0wndo3s.windows;

import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import com.sun.jna.platform.win32.WinUser;
import com.sun.jna.platform.win32.WinUser.MSG;

public class WindowsHotkeyService {

    public interface HotkeyCallback {
        void onToggle();
    }

    private final HotkeyCallback callback;
    private Thread loopThread;
    private volatile boolean running = false;

    private static final int HOTKEY_ID = 0xBEEF;
    private static final int MOD_CONTROL = 0x0002;
    private static final int VK_SPACE    = 0x20;

    public WindowsHotkeyService(HotkeyCallback callback) {
        this.callback = callback;
    }

    public void start() {
        if (running) return;
        running = true;

        loopThread = new Thread(() -> {
            boolean ok = User32.INSTANCE.RegisterHotKey((HWND) null, HOTKEY_ID, MOD_CONTROL, VK_SPACE);
            if (!ok) { running = false; return; }

            MSG msg = new MSG();
            while (running) {
                int result = User32.INSTANCE.GetMessage(msg, null, 0, 0);
                if (result == 0) break;      // WM_QUIT
                if (result == -1) break;     // erreur
            
                if (msg.message == WinUser.WM_HOTKEY && msg.wParam.intValue() == HOTKEY_ID) {
                    if (callback != null) callback.onToggle();
                } else {
                    User32.INSTANCE.TranslateMessage(msg);
                    User32.INSTANCE.DispatchMessage(msg);
                }
            }


            User32.INSTANCE.UnregisterHotKey(null, HOTKEY_ID);
        }, "HotkeyLoop");
        loopThread.setDaemon(true);
        loopThread.start();
    }

    public void stop() {
        running = false;
        try { User32.INSTANCE.PostQuitMessage(0); } catch (Throwable ignored) {}
    }
}
