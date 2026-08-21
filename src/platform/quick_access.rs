//! Pins the managed folders to the Windows Explorer Quick Access list.
//!
//! Quick Access is only reachable through the shell automation object, so every
//! operation goes through a short PowerShell script.

// Most of this surface is only reachable on Windows.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Folder ID of the Quick Access shell namespace.
const QUICK_ACCESS_NAMESPACE: &str = "shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}";

/// Pins `item` to Quick Access, returning whether it is pinned afterwards.
pub fn pin(item: &Path) -> bool {
    if !cfg!(windows) {
        return false;
    }
    if is_pinned(item) {
        return true;
    }
    invoke_verb_on_parent(item, "pintohome")
}

/// Lists the paths currently shown in Quick Access.
pub fn list_items() -> Vec<String> {
    let script = [
        "$sh = New-Object -ComObject Shell.Application",
        &format!("$qa = $sh.Namespace('{QUICK_ACCESS_NAMESPACE}')"),
        "$qa.Items() | ForEach-Object { $_.Path }",
    ]
    .join(" ; ");

    run_powershell_capture(&script)
}

/// Whether `item` already appears in Quick Access.
pub fn is_pinned(item: &Path) -> bool {
    let wanted = absolute(item);
    let wanted = wanted.to_string_lossy();
    list_items().iter().any(|listed| listed.eq_ignore_ascii_case(&wanted))
}

/// Clears the Quick Access history and restarts Explorer to apply it.
pub fn reset() -> bool {
    if !cfg!(windows) {
        return false;
    }

    let history_cleared = std::fs::remove_file(automatic_destinations_path()).is_ok();
    let explorer_restarted =
        run_command("taskkill /f /im explorer.exe") && run_command("start explorer.exe");

    history_cleared && explorer_restarted
}

/// Runs a shell context menu verb on `item` through its parent folder.
fn invoke_verb_on_parent(item: &Path, verb: &str) -> bool {
    let absolute = absolute(item);
    let (Some(parent), Some(name)) = (absolute.parent(), absolute.file_name()) else {
        return false;
    };

    let script = [
        "$ErrorActionPreference='Stop'".to_string(),
        "$sh = New-Object -ComObject Shell.Application".to_string(),
        format!("$p = {}", quote(&parent.to_string_lossy())),
        format!("$n = {}", quote(&name.to_string_lossy())),
        "$f = $sh.NameSpace($p)".to_string(),
        "if ($f -eq $null) { exit 2 }".to_string(),
        "$it = $f.ParseName($n)".to_string(),
        "if ($it -eq $null) { exit 3 }".to_string(),
        format!("$it.InvokeVerb({})", quote(verb)),
    ]
    .join(" ; ");

    // The verb reports success through exit code 0, and 1 when the item was
    // already pinned, which is just as good for our purposes.
    matches!(run_powershell(&script), Some(0 | 1))
}

fn absolute(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Escapes a value for a single quoted PowerShell string.
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn automatic_destinations_path() -> PathBuf {
    let profile = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(crate::core::home_dir);

    profile
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Recent")
        .join("AutomaticDestinations")
        .join("f01b4d95cf55d32a.automaticDestinations-ms")
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Option<i32> {
    use std::process::Command;

    Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
        .ok()
        .and_then(|output| output.status.code())
}

#[cfg(windows)]
fn run_powershell_capture(script: &str) -> Vec<String> {
    use std::process::Command;

    let Ok(output) = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
    else {
        return Vec::new();
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(windows)]
fn run_command(command_line: &str) -> bool {
    use std::process::Command;

    Command::new("cmd.exe")
        .args(["/d", "/c", command_line])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn run_powershell(_script: &str) -> Option<i32> {
    None
}

#[cfg(not(windows))]
fn run_powershell_capture(_script: &str) -> Vec<String> {
    Vec::new()
}

#[cfg(not(windows))]
fn run_command(_command_line: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_quotes_are_doubled_for_powershell() {
        assert_eq!(quote(r"C:\Users\me"), r"'C:\Users\me'");
        assert_eq!(quote("it's"), "'it''s'");
    }
}
