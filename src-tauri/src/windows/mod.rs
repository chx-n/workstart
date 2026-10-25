//! Windows adapter contract for Task 5. Discovery returns canonical executables,
//! never commands. `unresolved_defaults` supplies stable picker candidates when a
//! default is absent. Fixture callbacks stand in for Shell link resolution and
//! file canonicalization. Native entry points are available only on Windows; no
//! shared platform trait.

use crate::domain::{ErrorCategory, RunningState};
use std::collections::HashSet;

#[cfg(windows)]
pub mod native;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    UserAppPaths,
    MachineAppPaths,
    UserUninstall,
    MachineUninstall,
    UserShortcut,
    CommonShortcut,
    KnownLocation,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub source: Source,
    pub name: String,
    pub executable: Option<String>,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub name: String,
    pub executable: String,
    pub source: Source,
}

/// A known default with no resolved executable. Task 5 can forward this to the
/// settings UI, which may ask the user to choose an executable path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedApplication {
    pub name: String,
}

/// Normalize Windows paths independently of the host OS. Registry quoting is
/// accepted only around an entire path; command tails, devices and ADS are not.
pub fn executable_path(raw: &str) -> Option<String> {
    let value = raw.trim();
    let value = if value.starts_with('"') && value.ends_with('"') && value.len() > 2 {
        &value[1..value.len() - 1]
    } else {
        value
    };
    let value = value.replace('/', "\\");
    let value = value.strip_prefix(r"\\?\").unwrap_or(&value);
    let bytes = value.as_bytes();
    if bytes.len() < 7
        || !bytes[0].is_ascii_alphabetic()
        || &bytes[1..3] != b":\\"
        || value.chars().any(char::is_control)
        || !value.to_ascii_lowercase().ends_with(".exe")
    {
        return None;
    }
    if !value[3..].split('\\').all(|p| {
        !p.is_empty()
            && p != "."
            && p != ".."
            && !p.ends_with(['.', ' '])
            && !p.contains(['<', '>', ':', '"', '|', '?', '*'])
    }) {
        return None;
    }
    Some(value.into())
}

fn path_key(path: &str) -> Option<String> {
    executable_path(path).map(|p| p.to_lowercase())
}

fn known_identity(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "brave" | "brave.exe" | "brave browser" => Some("Brave"),
        "antigravity" | "antigravity.exe" | "antigravity ide" | "antigravity (user)" => {
            Some("Antigravity")
        }
        _ => None,
    }
}

fn basename(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

pub fn discover(
    mut records: Vec<Record>,
    locations: &[Record],
    mut resolve_shortcut: impl FnMut(&str) -> Option<String>,
    mut canonical_file: impl FnMut(&str) -> Option<String>,
) -> Vec<Application> {
    records.extend_from_slice(locations);
    records.sort_by_key(|r| r.source);
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    let mut result = Vec::new();
    for record in records {
        let candidate = record
            .executable
            .or_else(|| record.shortcut.as_deref().and_then(&mut resolve_shortcut));
        let Some(path) = candidate
            .as_deref()
            .and_then(executable_path)
            .and_then(|p| canonical_file(&p))
            .as_deref()
            .and_then(executable_path)
        else {
            continue;
        };
        let name = record.name.trim();
        if name.is_empty() || name.len() > 128 || name.chars().any(char::is_control) {
            continue;
        }
        let name = known_identity(basename(&path))
            .or_else(|| known_identity(name))
            .unwrap_or(name);
        let key = path.to_lowercase();
        let name_key = name.to_lowercase();
        if paths.contains(&key) || names.contains(&name_key) {
            continue;
        }
        paths.insert(key);
        names.insert(name_key);
        result.push(Application {
            name: name.into(),
            executable: path,
            source: record.source,
        });
    }
    result
}

pub fn resolve_default<'a>(apps: &'a [Application], name: &str) -> Option<&'a Application> {
    let identity = known_identity(name)?;
    apps.iter()
        .find(|a| known_identity(&a.name) == Some(identity))
}

pub fn unresolved_defaults(apps: &[Application]) -> Vec<UnresolvedApplication> {
    ["Brave", "Antigravity"]
        .into_iter()
        .filter(|name| resolve_default(apps, name).is_none())
        .map(|name| UnresolvedApplication { name: name.into() })
        .collect()
}

pub fn known_locations(
    local: Option<&str>,
    program_files: Option<&str>,
    program_files_x86: Option<&str>,
) -> Vec<Record> {
    let mut records = Vec::new();
    for root in [local, program_files, program_files_x86]
        .into_iter()
        .flatten()
    {
        for (name, suffix) in [
            (
                "Brave",
                r"BraveSoftware\Brave-Browser\Application\brave.exe",
            ),
            ("Antigravity", r"Antigravity\Antigravity.exe"),
            ("Antigravity", r"Programs\Antigravity\Antigravity.exe"),
        ] {
            records.push(Record {
                source: Source::KnownLocation,
                name: name.into(),
                executable: Some(format!("{}\\{suffix}", root.trim_end_matches('\\'))),
                shortcut: None,
            });
        }
    }
    records
}

/// DisplayIcon is a path plus an optional numeric resource index, never a
/// command. InstallLocation is usable only for applications with known layouts.
pub fn uninstall_executable(
    name: &str,
    display_icon: Option<&str>,
    install_location: Option<&str>,
) -> Option<String> {
    if let Some(icon) = display_icon {
        let path = icon
            .rsplit_once(',')
            .filter(|(_, index)| index.trim().parse::<i32>().is_ok())
            .map_or(icon, |(path, _)| path);
        if let Some(path) = executable_path(path) {
            return Some(path);
        }
    }
    let suffix = match known_identity(name)? {
        "Brave" => r"Application\brave.exe",
        "Antigravity" => "Antigravity.exe",
        _ => return None,
    };
    executable_path(&format!(
        "{}\\{suffix}",
        install_location?.trim_end_matches('\\')
    ))
}

/// A checked plan with private fields, consumable by native::launch. Only the
/// backend should prepare it after resolving a stored target ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectLaunch {
    executable: String,
    arguments: Vec<String>,
}
impl DirectLaunch {
    pub fn executable(&self) -> &str {
        &self.executable
    }
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }
}
pub fn prepare_launch(path: &str, arguments: &[String]) -> Result<DirectLaunch, ErrorCategory> {
    let executable = executable_path(path).ok_or(ErrorCategory::InvalidConfiguration)?;
    if arguments.len() > 64 || arguments.iter().any(|a| a.len() > 4096 || a.contains('\0')) {
        return Err(ErrorCategory::InvalidConfiguration);
    }
    Ok(DirectLaunch {
        executable,
        arguments: arguments.to_vec(),
    })
}

#[derive(Debug, Clone)]
pub struct Process {
    pub canonical_path: Option<String>,
    pub name: String,
}

/// `complete` is false on snapshot/permission/canonicalization failures. A known
/// alias is used only when that process has no readable path; a conflicting
/// authoritative path must never be overridden by its basename.
pub fn running_state(executable: &str, processes: &[Process], complete: bool) -> RunningState {
    let Some(target) = path_key(executable) else {
        return RunningState::Unknown;
    };
    if processes
        .iter()
        .any(|p| p.canonical_path.as_deref().and_then(path_key).as_ref() == Some(&target))
    {
        return RunningState::Running;
    }
    let alias = known_identity(basename(executable));
    if alias.is_some()
        && processes
            .iter()
            .any(|p| p.canonical_path.is_none() && known_identity(&p.name) == alias)
    {
        return RunningState::Running;
    }
    if !complete
        || processes.iter().any(|p| {
            p.canonical_path.is_none() && p.name.eq_ignore_ascii_case(basename(executable))
        })
    {
        RunningState::Unknown
    } else {
        RunningState::NotRunning
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaIcon {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Icon {
    Generic,
    Rgba(RgbaIcon),
}
pub fn lookup_icon(path: &str, mut extract: impl FnMut(&str) -> Option<RgbaIcon>) -> Icon {
    executable_path(path)
        .and_then(|p| extract(&p))
        .filter(|i| {
            i.width > 0
                && i.height > 0
                && i.width <= 256
                && i.height <= 256
                && i.pixels.len() == i.width as usize * i.height as usize * 4
        })
        .map(Icon::Rgba)
        .unwrap_or(Icon::Generic)
}
