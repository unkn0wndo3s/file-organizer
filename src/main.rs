mod core;
mod platform;
mod ui;

use crate::core::{folders, home_dir, log_bus, mover};
use crate::platform::autostart;
use crate::platform::hotkey::HotkeyService;
use crate::platform::quick_access;
use crate::platform::tray::{Tray, TrayCommand};
use crate::ui::icons::{IconName, LocalAssets};
use crate::ui::list::List;
use crate::ui::log_console::LogConsole;
use crate::ui::title_bar::{TitleBar, TitleBarEvent};
use crossbeam_channel::{unbounded, Receiver, Sender};
use gpui::*;
use gpui_component::{input::*, Icon, Root, StyledExt};
use notify::{Event, RecursiveMode, Watcher};
use std::sync::Arc;
use std::time::Duration;

/// How often the foreground task drains the background channels.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

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
    title_bar: Entity<TitleBar>,
    list: Entity<List>,
    log_console: Entity<LogConsole>,
    input_state: Entity<InputState>,
    /// The watcher stops reporting as soon as it is dropped, so it is kept
    /// alive for as long as the application runs.
    _watcher: Option<notify::RecommendedWatcher>,
    _tray: Tray,
    _hotkey: HotkeyService,
}

impl Main {
    pub fn new(
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

        let console_target = log_console.clone();
        cx.subscribe(&title_bar, move |_this, _title_bar, event: &TitleBarEvent, cx| match event {
            TitleBarEvent::ToggleConsole => console_target.update(cx, |console, cx| console.toggle(cx)),
        })
        .detach();

        let (commands_tx, commands_rx) = unbounded::<AppCommand>();
        let tray = install_tray(commands_tx.clone());
        let hotkey = start_hotkey(commands_tx);

        let (watcher, fs_events) = start_watcher();
        Self::spawn_event_pump(fs_events, log_lines, commands_rx, &list, &log_console, cx);
        Self::spawn_startup_sweep(&list, cx);

        Self {
            window_handle: window.window_handle(),
            title_bar,
            input_state,
            list,
            log_console,
            _watcher: watcher,
            _tray: tray,
            _hotkey: hotkey,
        }
    }

    /// Drains the background channels on the foreground thread, which is the
    /// only place the gpui entities may be updated from.
    fn spawn_event_pump(
        fs_events: Receiver<Event>,
        log_lines: Receiver<String>,
        commands: Receiver<AppCommand>,
        list: &Entity<List>,
        log_console: &Entity<LogConsole>,
        cx: &mut Context<Self>,
    ) {
        let list = list.downgrade();
        let log_console = log_console.downgrade();
        let executor = cx.background_executor().clone();

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    executor.timer(POLL_INTERVAL).await;

                    let events: Vec<Event> = fs_events.try_iter().collect();
                    if !events.is_empty() {
                        let updated = list.update(&mut cx, |list, cx| {
                            for event in events {
                                list.handle_fs_event(event, cx);
                            }
                        });
                        if updated.is_err() {
                            break;
                        }
                    }

                    let lines: Vec<String> = log_lines.try_iter().collect();
                    if !lines.is_empty() {
                        let updated = log_console.update(&mut cx, |console, cx| {
                            for line in lines {
                                console.append(line, cx);
                            }
                        });
                        if updated.is_err() {
                            break;
                        }
                    }

                    for command in commands.try_iter() {
                        if this.update(&mut cx, |this, cx| this.handle_command(command, cx)).is_err() {
                            return;
                        }
                    }
                }
            }
        })
        .detach();
    }

    /// Sweeps Downloads into the home buckets, then rebuilds the index, exactly
    /// as the Java application did on startup.
    fn spawn_startup_sweep(list: &Entity<List>, cx: &mut Context<Self>) {
        let list = list.downgrade();
        let executor = cx.background_executor().clone();

        cx.spawn(move |_this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let moved = executor
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

                        let moved = mover::sweep(&home, &downloads);
                        log_bus::log(format!("[scan] moved={moved}"));

                        moved
                    })
                    .await;

                if moved > 0 {
                    let _ = list.update(&mut cx, |list, cx| list.reload("startup sweep", cx));
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
                self.log_console.update(cx, |console, cx| console.toggle(cx));
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
        self.with_window(cx, |window| {
            window.activate_window();
            log_bus::log("[search] window shown");
        });
    }

    /// gpui has no dedicated hide, so the window is minimized instead, which is
    /// what the JavaFX stage did when it stepped out of the way.
    fn hide_window(&self, cx: &mut Context<Self>) {
        self.with_window(cx, |window| {
            window.minimize_window();
            log_bus::log("[search] window hidden");
        });
    }

    fn toggle_window(&self, cx: &mut Context<Self>) {
        self.with_window(cx, |window| {
            if window.is_window_active() {
                log_bus::log("[search] toggle: hiding the window");
                window.minimize_window();
            } else {
                log_bus::log("[search] toggle: showing the window");
                window.activate_window();
            }
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
            .bg(rgb(0x1e1e1e))
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
            let _ = tx.send(event);
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
        let _ = commands.send(command);
    }))
}

fn start_hotkey(commands: Sender<AppCommand>) -> HotkeyService {
    let mut hotkey = HotkeyService::new(Arc::new(move || {
        let _ = commands.send(AppCommand::ToggleWindow);
    }))
    .on_registration(Arc::new(|result| match result {
        Ok(name) => log_bus::log(format!("[hotkey] {name} registered")),
        Err(reason) => log_bus::log(format!("[hotkey:error] registration failed: {reason}")),
    }));

    hotkey.start();
    hotkey
}

/// Bridges the log bus into the console view, which lives on the foreground thread.
fn capture_log_lines() -> Receiver<String> {
    let (tx, rx) = unbounded::<String>();
    log_bus::add_listener(move |line| {
        let _ = tx.send(line.to_string());
    });
    rx
}

/// Handles the flags the installers and the packaging scripts use, and reports
/// whether the graphical application should still start.
fn run_cli(arguments: &[String]) -> Option<i32> {
    let Some(flag) = arguments.first() else { return None };

    let result = match flag.as_str() {
        "--enable-autostart" => autostart::enable().map(|()| "autostart enabled".to_string()),
        "--disable-autostart" => autostart::disable().map(|()| "autostart disabled".to_string()),
        "--autostart-status" => {
            Ok(format!("autostart is {}", if autostart::is_enabled() { "enabled" } else { "disabled" }))
        }
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

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = run_cli(&arguments) {
        std::process::exit(code);
    }

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
            let screen_bounds = cx.update(|cx| {
                cx.displays().first().map(|display| display.bounds()).unwrap_or(Bounds::default())
            })?;

            let width = screen_bounds.size.width * 0.6;
            let height = screen_bounds.size.height * 0.6;
            let x = screen_bounds.origin.x + (screen_bounds.size.width - width) / 2.0;
            let y = screen_bounds.origin.y + (screen_bounds.size.height - height) / 2.0;

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(point(x, y), size(width, height)))),
                is_resizable: true,
                titlebar: None,
                ..Default::default()
            };

            cx.open_window(options, |window, cx| {
                let title_bar = cx.new(|_cx| TitleBar::new());
                let list = cx.new(List::new);
                let log_console = cx.new(|_cx| LogConsole::new());
                let view = cx.new(|cx| Main::new(title_bar, list, log_console, log_lines, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
