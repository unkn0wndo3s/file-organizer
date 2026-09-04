mod core;
mod platform;
mod ui;

use crate::core::{folders, home_dir, log_bus, mover};
use crate::platform::autostart;
use crate::platform::hotkey::HotkeyService;
use crate::platform::ipc::IpcServer;
use crate::platform::quick_access;
use crate::platform::tray::{Tray, TrayCommand};
use crate::ui::icons::{IconName, LocalAssets};
use crate::ui::list::List;
use crate::ui::log_console::LogConsole;
use crate::ui::title_bar::{TitleBar, TitleBarEvent};
use async_channel::{unbounded, Receiver, Sender};
use gpui::*;
use gpui_component::{input::*, Icon, Root, StyledExt};
use notify::{Event, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Rows shown at once, the window is sized to fit exactly this many.
const VISIBLE_RESULTS: usize = 5;
/// Fraction of the screen's width the window occupies.
const WIDTH_RATIO: f32 = 2.0 / 5.0;
/// Everything above the result rows: the title bar, the search input, and
/// their paddings and gaps.
const CHROME_HEIGHT: f32 = 148.0;
/// Extra height the window grows by while the console panel is open: its own
/// height (`log_console`'s `h_64`) plus the gap before it.
const CONSOLE_HEIGHT: f32 = 256.0 + 16.0;

/// Requests raised by the tray and the hotkey, which run on their own threads
/// and cannot touch the gpui context directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppCommand {
    ShowWindow,
    HideWindow,
    ToggleWindow,
    ToggleConsole,
    ToggleAutostart,
    Quit,
}

pub struct Main {
    /// Needed to drive the window from the tray and hotkey threads, which only
    /// reach the application context.
    window_handle: AnyWindowHandle,
    /// Tracks our own show/hide state rather than reading it back from the
    /// window: a visible-but-unfocused window (the user clicked elsewhere) is
    /// still "shown" as far as the app is concerned, but `is_window_active()`
    /// would report it the same as a minimized one, making a toggle re-show
    /// it instead of hiding it.
    shown: Arc<AtomicBool>,
    title_bar: Entity<TitleBar>,
    list: Entity<List>,
    log_console: Entity<LogConsole>,
    input_state: Entity<InputState>,
    /// The watcher stops reporting as soon as it is dropped, so it is kept
    /// alive for as long as the application runs.
    _watcher: Option<notify::RecommendedWatcher>,
    _tray: Tray,
    _hotkey: HotkeyService,
    _ipc: IpcServer,
}

impl Main {
    pub fn new(
        shown: Arc<AtomicBool>,
        title_bar: Entity<TitleBar>,
        list: Entity<List>,
        log_console: Entity<LogConsole>,
        log_lines: Receiver<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder("Search for file or folder", window, cx);
            state
        });

        let search_target = list.clone();
        cx.subscribe(&input_state, move |_this, input, _event: &gpui_component::input::InputEvent, cx| {
            let query = input.read(cx).value().to_string();
            search_target.update(cx, |list, cx| list.set_search(query, cx));
        })
        .detach();

        cx.subscribe(&title_bar, move |this, _title_bar, event: &TitleBarEvent, cx| match event {
            TitleBarEvent::ToggleConsole => this.toggle_console(cx),
        })
        .detach();

        let (commands_tx, commands_rx) = unbounded::<AppCommand>();
        let tray = install_tray(commands_tx.clone());
        let hotkey = start_hotkey(commands_tx.clone());
        let ipc = start_ipc_server(commands_tx);

        let (watcher, fs_events) = start_watcher();
        Self::spawn_event_pump(fs_events, log_lines, commands_rx, &list, &log_console, cx);
        Self::spawn_startup_sweep(&list, cx);

        Self {
            window_handle: window.window_handle(),
            shown,
            title_bar,
            input_state,
            list,
            log_console,
            _watcher: watcher,
            _tray: tray,
            _hotkey: hotkey,
            _ipc: ipc,
        }
    }

    /// Forwards each background channel onto the foreground thread, which is
    /// the only place the gpui entities may be updated from.
    ///
    /// Every pump awaits its channel rather than polling it on a timer, so an
    /// idle application wakes up for nothing.
    fn spawn_event_pump(
        fs_events: Receiver<Event>,
        log_lines: Receiver<String>,
        commands: Receiver<AppCommand>,
        list: &Entity<List>,
        log_console: &Entity<LogConsole>,
        cx: &mut Context<Self>,
    ) {
        let list = list.downgrade();
        cx.spawn(move |_this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Ok(event) = fs_events.recv().await {
                    // Drain whatever else arrived in the meantime, so a burst
                    // of events costs one redraw rather than one each.
                    let updated = list.update(&mut cx, |list, cx| {
                        let mut changed = list.handle_fs_event(event);
                        while let Ok(event) = fs_events.try_recv() {
                            changed |= list.handle_fs_event(event);
                        }
                        if changed {
                            cx.notify();
                        }
                    });

                    if updated.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        let log_console = log_console.downgrade();
        cx.spawn(move |_this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Ok(line) = log_lines.recv().await {
                    let updated = log_console.update(&mut cx, |console, cx| {
                        console.append(line, cx);
                        while let Ok(line) = log_lines.try_recv() {
                            console.append(line, cx);
                        }
                    });

                    if updated.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Ok(command) = commands.recv().await {
                    if this.update(&mut cx, |this, cx| this.handle_command(command, cx)).is_err() {
                        break;
                    }
                }
            }
        })
        .detach();
    }

    /// Sweeps Downloads into the home buckets, then rebuilds the index, exactly
    /// as the Java application did on startup.
    ///
    /// Everything that touches the disk runs on the background executor, so a
    /// slow or crowded home folder never stalls the window.
    fn spawn_startup_sweep(list: &Entity<List>, cx: &mut Context<Self>) {
        let list = list.downgrade();
        let executor = cx.background_executor().clone();

        cx.spawn(move |_this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let rescanned = executor
                    .spawn(async move {
                        let home = home_dir();

                        for folder in folders::managed_paths(&home) {
                            let pinned = quick_access::pin(&folder);
                            log_bus::log(format!(
                                "[pin] {} {}",
                                if pinned { "OK" } else { "KO" },
                                folder.display()
                            ));
                        }

                        let downloads = home.join(folders::DOWNLOADS);
                        log_bus::log(format!("[scan] start {}", downloads.display()));

                        // Downloads is an inbox: a loose file is filed by type,
                        // and a loose folder is clutter to be tidied into
                        // Folders just the same.
                        let mut moved = mover::sweep(&home, &downloads);

                        // Every other managed folder is already organized, so
                        // only a misplaced *file* is relocated there — a
                        // subfolder is left exactly where the user put it.
                        for folder in folders::MANAGED {
                            if folder == folders::DOWNLOADS {
                                continue;
                            }
                            moved += mover::resort(&home, &home.join(folder));
                        }

                        log_bus::log(format!("[scan] moved={moved}"));

                        // The sweep created the buckets and filled them, so the
                        // index built before it ran is already out of date.
                        (moved > 0).then(List::load_items)
                    })
                    .await;

                if let Some(items) = rescanned {
                    let _ = list.update(&mut cx, |list, cx| list.apply_items(items, "startup sweep", cx));
                }
            }
        })
        .detach();
    }

    fn handle_command(&mut self, command: AppCommand, cx: &mut Context<Self>) {
        match command {
            AppCommand::ShowWindow => self.show_window(cx),
            AppCommand::HideWindow => self.hide_window(cx),
            AppCommand::ToggleWindow => self.toggle_window(cx),
            AppCommand::ToggleConsole => {
                self.show_window(cx);
                self.toggle_console(cx);
            }
            AppCommand::ToggleAutostart => {
                autostart::toggle();
            }
            AppCommand::Quit => {
                log_bus::log("[app] quit requested");
                cx.quit();
            }
        }
    }

    fn show_window(&self, cx: &mut Context<Self>) {
        self.shown.store(true, Ordering::Relaxed);
        self.with_window(cx, |window| {
            window.activate_window();
            log_bus::log("[search] window shown");
        });
    }

    /// gpui has no dedicated hide, so the window is minimized instead, which is
    /// what the JavaFX stage did when it stepped out of the way.
    fn hide_window(&self, cx: &mut Context<Self>) {
        self.shown.store(false, Ordering::Relaxed);
        self.with_window(cx, |window| {
            window.minimize_window();
            log_bus::log("[search] window hidden");
        });
    }

    fn toggle_window(&self, cx: &mut Context<Self>) {
        if self.shown.load(Ordering::Relaxed) {
            self.hide_window(cx);
        } else {
            self.show_window(cx);
        }
    }

    /// The window is sized to fit the result list exactly, with no room to
    /// spare, so the console panel needs the window itself to grow into
    /// rather than sharing that space, or its content would spill past the
    /// bottom edge.
    fn toggle_console(&self, cx: &mut Context<Self>) {
        let visible = self.log_console.update(cx, |console, cx| {
            console.toggle(cx);
            console.is_visible()
        });

        self.with_window(cx, |window| {
            let mut bounds = window.bounds();
            bounds.size.height =
                if visible { bounds.size.height + px(CONSOLE_HEIGHT) } else { bounds.size.height - px(CONSOLE_HEIGHT) };
            window.resize(bounds.size);
        });
    }

    fn with_window(&self, cx: &mut Context<Self>, action: impl FnOnce(&mut Window)) {
        let _ = self.window_handle.update(cx, |_, window, _| action(window));
    }
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .rounded_lg()
            .bg(rgba(0x1e1e1ee6))
            .child(self.title_bar.clone())
            .child(
                div()
                    .flex_grow()
                    .v_flex()
                    .p_4()
                    .gap_4()
                    .child(
                        Input::new(&self.input_state)
                            .w_full()
                            .prefix(Icon::new(IconName::Search).size_4().text_color(rgb(0x777777))),
                    )
                    .child(
                        div()
                            .w_full()
                            .h_0()
                            .flex_grow()
                            .id("list-scroll-view")
                            .overflow_y_scroll()
                            .child(self.list.clone()),
                    )
                    .child(self.log_console.clone()),
            )
    }
}

/// Watches the managed folders so the index follows what happens on disk.
fn start_watcher() -> (Option<notify::RecommendedWatcher>, Receiver<Event>) {
    let (tx, rx): (Sender<Event>, Receiver<Event>) = unbounded();

    let mut watcher = match notify::recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result {
            let _ = tx.try_send(event);
        }
    }) {
        Ok(watcher) => watcher,
        Err(error) => {
            log_bus::log(format!("[watch:init:error] {error}"));
            return (None, rx);
        }
    };

    for root in List::indexed_roots() {
        if !root.is_dir() {
            continue;
        }
        match watcher.watch(&root, RecursiveMode::NonRecursive) {
            Ok(()) => log_bus::log(format!("[watch] {}", root.display())),
            Err(error) => log_bus::log(format!("[watch:error] {} : {error}", root.display())),
        }
    }

    (Some(watcher), rx)
}

fn install_tray(commands: Sender<AppCommand>) -> Tray {
    Tray::install(Arc::new(move |command: TrayCommand| {
        let command = match command {
            TrayCommand::Open => AppCommand::ShowWindow,
            TrayCommand::Hide => AppCommand::HideWindow,
            TrayCommand::Toggle => AppCommand::ToggleWindow,
            TrayCommand::ToggleConsole => AppCommand::ToggleConsole,
            TrayCommand::ToggleAutostart => AppCommand::ToggleAutostart,
            TrayCommand::Quit => AppCommand::Quit,
        };
        let _ = commands.try_send(command);
    }))
}

fn start_ipc_server(commands: Sender<AppCommand>) -> IpcServer {
    IpcServer::start(move || {
        let _ = commands.try_send(AppCommand::ToggleWindow);
    })
}

fn start_hotkey(commands: Sender<AppCommand>) -> HotkeyService {
    let mut hotkey = HotkeyService::new(Arc::new(move || {
        let _ = commands.try_send(AppCommand::ToggleWindow);
    }))
    .on_registration(Arc::new(|result| match result {
        Ok(name) => log_bus::log(format!("[hotkey] {name} registered")),
        Err(reason) => log_bus::log(format!("[hotkey:error] registration failed: {reason}")),
    }));

    hotkey.start();
    hotkey
}

/// The displays a Wayland client learns about are discovered asynchronously,
/// over the registry the compositor sends after the connection is made, so
/// `cx.displays()` can still be empty in the first moments after startup.
/// Polls briefly rather than risking a 0x0 window on whichever run loses
/// that race.
async fn wait_for_a_display(cx: &mut AsyncApp) -> anyhow::Result<Bounds<Pixels>> {
    const ATTEMPTS: u32 = 50;
    const INTERVAL: Duration = Duration::from_millis(20);

    for attempt in 0..ATTEMPTS {
        let bounds = cx.update(|cx| cx.displays().first().map(|display| display.bounds()))?;
        if let Some(bounds) = bounds {
            return Ok(bounds);
        }

        if attempt == 0 {
            log_bus::log("[window] waiting for the compositor to report a display...");
        }
        cx.background_executor().timer(INTERVAL).await;
    }

    log_bus::log("[window:warning] no display reported after 1s, falling back to a default size");
    Ok(Bounds::new(point(px(0.0), px(0.0)), size(px(1536.0), px(864.0))))
}

/// Bridges the log bus into the console view, which lives on the foreground thread.
fn capture_log_lines() -> Receiver<String> {
    let (tx, rx) = unbounded::<String>();
    log_bus::add_listener(move |line| {
        let _ = tx.try_send(line.to_string());
    });
    rx
}

/// Handles the flags the installers and the packaging scripts use, and reports
/// whether the graphical application should still start.
fn run_cli(arguments: &[String]) -> Option<i32> {
    let flag = arguments.first()?;

    let result: Result<String, String> = match flag.as_str() {
        "--enable-autostart" => {
            autostart::enable().map(|()| "autostart enabled".to_string()).map_err(|error| error.to_string())
        }
        "--disable-autostart" => {
            autostart::disable().map(|()| "autostart disabled".to_string()).map_err(|error| error.to_string())
        }
        "--autostart-status" => {
            Ok(format!("autostart is {}", if autostart::is_enabled() { "enabled" } else { "disabled" }))
        }
        "--toggle" => platform::ipc::send_toggle().map(|()| "toggled".to_string()),
        "--version" => Ok(format!("file-organizer {}", env!("CARGO_PKG_VERSION"))),
        "--help" | "-h" => {
            println!(
                "file-organizer {}\n\n\
                 Usage: file-organizer [OPTION]\n\n\
                 With no option, the graphical application starts.\n\n\
                 Options:\n\
                 \x20 --enable-autostart   start with the user's session\n\
                 \x20 --disable-autostart  stop starting with the session\n\
                 \x20 --autostart-status   report the current setting\n\
                 \x20 --toggle             show or hide a running instance's window\n\
                 \x20                      (bind this to a key in your compositor or\n\
                 \x20                      window manager where the global Ctrl+Space\n\
                 \x20                      shortcut cannot reach the application)\n\
                 \x20 --version            print the version\n\
                 \x20 -h, --help           print this help",
                env!("CARGO_PKG_VERSION")
            );
            return Some(0);
        }
        unknown => {
            eprintln!("file-organizer: unknown option {unknown}, try --help");
            return Some(2);
        }
    };

    match result {
        Ok(message) => {
            println!("{message}");
            Some(0)
        }
        Err(error) => {
            eprintln!("file-organizer: {error}");
            Some(1)
        }
    }
}

/// Makes gpui pick its X11 backend over its native Wayland one, when both are
/// available, by hiding `WAYLAND_DISPLAY` from it.
///
/// Wayland's `xdg-shell` has a request to minimize a window but none to
/// restore it, so a minimized native Wayland window can only be brought back
/// by the user, never by the application; and Wayland deliberately keeps
/// global shortcuts away from applications entirely, which is why `Ctrl+Space`
/// only reaches XWayland windows (see `platform::hotkey`). Both the tray's
/// hide/show and the global shortcut need a real window to attach to, so
/// running through XWayland, which supports both, is preferred over running
/// natively whenever an X server is actually reachable.
fn prefer_xwayland() {
    if std::env::var_os("DISPLAY").is_some() && std::env::var_os("WAYLAND_DISPLAY").is_some() {
        log_bus::log("[window] an X server is available, preferring XWayland over native Wayland");
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = run_cli(&arguments) {
        std::process::exit(code);
    }

    prefer_xwayland();

    let log_lines = capture_log_lines();

    // The managed folders have to exist before anything else: the index reads
    // them and the watcher can only register a folder that is already there.
    if let Err(error) = folders::ensure_all(&home_dir()) {
        log_bus::log(format!("[init:error] {error}"));
    }

    let app = Application::new().with_assets(LocalAssets::new());

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            let screen_bounds = wait_for_a_display(cx).await?;

            // A launcher-style popup, not a full window: fixed size, wide
            // enough for a comfortable line of text, tall enough for exactly
            // `VISIBLE_RESULTS` rows, centered on the screen.
            let width = screen_bounds.size.width * WIDTH_RATIO;
            let height = crate::ui::list::ROW_HEIGHT * VISIBLE_RESULTS + px(CHROME_HEIGHT);
            let x = screen_bounds.origin.x + (screen_bounds.size.width - width) / 2.0;
            let y = screen_bounds.origin.y + (screen_bounds.size.height - height) / 2.0;
            let bounds = Bounds::new(point(x, y), size(width, height));

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                is_resizable: false,
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                // Without this, a non-maximized window gets the compositor's
                // own server side decorations — close/minimize buttons the
                // title bar deliberately does not have.
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            };

            let shown = Arc::new(AtomicBool::new(true));
            let shown_at_startup = Arc::clone(&shown);

            let window_handle = cx.open_window(options, |window, cx| {
                let title_bar = cx.new(|_cx| TitleBar::new());
                let list = cx.new(List::new);
                let log_console = cx.new(|_cx| LogConsole::new());
                let view = cx.new(|cx| Main::new(shown, title_bar, list, log_console, log_lines, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            cx.background_executor().timer(Duration::from_millis(50)).await;
            // The app lives in the tray until Ctrl+Space, the tray icon, or
            // `--toggle` brings it up; it should not appear on launch.
            let _ = window_handle.update(cx, |_root, window, _cx| window.minimize_window());
            shown_at_startup.store(false, Ordering::Relaxed);

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
