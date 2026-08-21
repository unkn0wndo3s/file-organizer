use chrono::Local;
use std::sync::{Mutex, OnceLock};

/// Callback invoked for every line published on the bus.
pub type Listener = Box<dyn Fn(&str) + Send + Sync + 'static>;

fn registry() -> &'static Mutex<Vec<Listener>> {
    static REGISTRY: OnceLock<Mutex<Vec<Listener>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Registers a listener that receives every timestamped log line.
pub fn add_listener<F>(listener: F)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    if let Ok(mut listeners) = registry().lock() {
        listeners.push(Box::new(listener));
    }
}

/// Publishes a line on stdout and to every registered listener.
pub fn log(line: impl AsRef<str>) {
    let message = format!("[{}] {}", Local::now().format("%H:%M:%S"), line.as_ref());
    println!("{message}");

    if let Ok(listeners) = registry().lock() {
        for listener in listeners.iter() {
            listener(&message);
        }
    }
}
