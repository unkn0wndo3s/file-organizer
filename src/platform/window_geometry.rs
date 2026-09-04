//! Locating the display currently in focus, and repositioning this process's
//! own window onto it.
//!
//! gpui has no public API to move a window after it is created, only to
//! resize it, and its own display list merges every monitor into a single
//! bounds on at least some multi-monitor X11 setups, which makes it useless
//! for telling which one the user is actually working on. Both directions go
//! through the platform natively instead of through gpui: the display under
//! the pointer is what "in focus" means here, and this process's own top
//! level window is found by matching its process id, since gpui's X11
//! backend cannot even report its own window handle (`HasWindowHandle` is
//! unimplemented there).

/// The bounds, in screen coordinates, of whichever display currently has the
/// pointer. `None` when that cannot be determined, in which case callers
/// should leave the window wherever it already is.
pub fn focused_display_bounds() -> Option<(i32, i32, i32, i32)> {
    imp::focused_display_bounds()
}

/// Moves this process's own top level window to `(x, y)`, screen relative.
pub fn move_own_window(x: i32, y: i32) -> Result<(), String> {
    imp::move_own_window(x, y)
}

#[cfg(target_os = "linux")]
mod imp {
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as _;
    use x11rb::protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt as _, Window};

    pub fn focused_display_bounds() -> Option<(i32, i32, i32, i32)> {
        let (connection, screen_index) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots[screen_index].root;

        let pointer = connection.query_pointer(root).ok()?.reply().ok()?;
        let (pointer_x, pointer_y) = (pointer.root_x as i32, pointer.root_y as i32);

        let monitors = connection.randr_get_monitors(root, true).ok()?.reply().ok()?;
        monitors.monitors.into_iter().find_map(|monitor| {
            let (x, y, width, height) =
                (monitor.x as i32, monitor.y as i32, monitor.width as i32, monitor.height as i32);
            let contains = pointer_x >= x && pointer_x < x + width && pointer_y >= y && pointer_y < y + height;
            contains.then_some((x, y, width, height))
        })
    }

    pub fn move_own_window(x: i32, y: i32) -> Result<(), String> {
        let (connection, screen_index) = x11rb::connect(None).map_err(|error| error.to_string())?;
        let root = connection.setup().roots[screen_index].root;
        let window = own_window(&connection, root)?;

        connection
            .configure_window(window, &ConfigureWindowAux::new().x(x).y(y))
            .map_err(|error| error.to_string())?;
        connection.flush().map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Finds this process's own top level window among `_NET_CLIENT_LIST` by
    /// matching `_NET_WM_PID`, since gpui cannot report its own window id.
    fn own_window(connection: &impl Connection, root: Window) -> Result<Window, String> {
        let client_list = intern(connection, b"_NET_CLIENT_LIST").ok_or("no _NET_CLIENT_LIST atom")?;
        let net_wm_pid = intern(connection, b"_NET_WM_PID").ok_or("no _NET_WM_PID atom")?;

        let reply = connection
            .get_property(false, root, client_list, AtomEnum::WINDOW, 0, 1024)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;

        let pid = std::process::id();
        reply
            .value32()
            .into_iter()
            .flatten()
            .find(|&window| window_pid(connection, window, net_wm_pid) == Some(pid))
            .ok_or_else(|| "this process's window was not found on _NET_CLIENT_LIST".to_string())
    }

    fn window_pid(connection: &impl Connection, window: Window, net_wm_pid: u32) -> Option<u32> {
        let reply =
            connection.get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1).ok()?.reply().ok()?;
        reply.value32()?.next()
    }

    fn intern(connection: &impl Connection, name: &[u8]) -> Option<u32> {
        connection.intern_atom(false, name).ok()?.reply().ok().map(|reply| reply.atom)
    }
}

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT};
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetCursorPos, GetWindowThreadProcessId, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    };

    pub fn focused_display_bounds() -> Option<(i32, i32, i32, i32)> {
        let mut point: POINT = unsafe { std::mem::zeroed() };
        if unsafe { GetCursorPos(&mut point) } == 0 {
            return None;
        }

        let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
        let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
            return None;
        }

        let rect = info.rcMonitor;
        Some((rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top))
    }

    pub fn move_own_window(x: i32, y: i32) -> Result<(), String> {
        let Some(window) = find_own_window() else {
            return Err("this process's window was not found".to_string());
        };

        let ok =
            unsafe { SetWindowPos(window, std::ptr::null_mut(), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) };
        if ok == 0 {
            return Err(format!("SetWindowPos failed: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }

    /// Enumerates every top level window until one owned by this process's
    /// own id turns up, since gpui cannot report its own window handle.
    fn find_own_window() -> Option<HWND> {
        struct Search {
            pid: u32,
            found: HWND,
        }

        unsafe extern "system" fn callback(window: HWND, lparam: LPARAM) -> i32 {
            let search = unsafe { &mut *(lparam as *mut Search) };
            let mut window_pid = 0u32;
            unsafe { GetWindowThreadProcessId(window, &mut window_pid) };
            if window_pid == search.pid {
                search.found = window;
                return 0;
            }
            1
        }

        let mut search = Search { pid: std::process::id(), found: std::ptr::null_mut() };
        unsafe { EnumWindows(Some(callback), std::ptr::addr_of_mut!(search) as LPARAM) };

        if search.found.is_null() {
            None
        } else {
            Some(search.found)
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    pub fn focused_display_bounds() -> Option<(i32, i32, i32, i32)> {
        None
    }

    pub fn move_own_window(_x: i32, _y: i32) -> Result<(), String> {
        Err("unsupported platform".to_string())
    }
}
