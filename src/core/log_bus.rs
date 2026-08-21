#![allow(dead_code)]

use chrono::Local;
use std::sync::{Mutex, OnceLock};

/// Callback invoked for every line published on the bus.
pub type Listener = Box<dyn Fn(&str) + Send + Sync + 'static>;

/// Identifies a registered listener so it can be removed later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListenerId(u64);

struct Registration {
    id: ListenerId,
    listener: Listener,
}

fn registry() -> &'static Mutex<Vec<Registration>> {
    static REGISTRY: OnceLock<Mutex<Vec<Registration>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn next_id() -> ListenerId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    ListenerId(COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Registers a listener that receives every timestamped log line.
pub fn add_listener<F>(listener: F) -> ListenerId
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let id = next_id();
    if let Ok(mut listeners) = registry().lock() {
        listeners.push(Registration { id, listener: Box::new(listener) });
    }
    id
}

/// Removes a previously registered listener.
pub fn remove_listener(id: ListenerId) {
    if let Ok(mut listeners) = registry().lock() {
        listeners.retain(|registration| registration.id != id);
    }
}

/// Publishes a line on stdout and to every registered listener.
pub fn log(line: impl AsRef<str>) {
    let message = format!("[{}] {}", Local::now().format("%H:%M:%S"), line.as_ref());
    println!("{message}");

    if let Ok(listeners) = registry().lock() {
        for registration in listeners.iter() {
            (registration.listener)(&message);
        }
    }
}
