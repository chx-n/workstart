use super::{AppIcon, InstalledApp, PlatformAdapter, ProcessSnapshot, ResolvedLaunch};
use crate::domain::{AppTarget, ErrorCategory, LaunchRef, Platform, ProcessMatcher, RunningState};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
mod desktop;
use desktop::DesktopEntry;

pub struct LinuxPaths {
    pub data_dirs: Vec<PathBuf>,
    pub executable_dirs: Vec<PathBuf>,
    pub icon_dirs: Vec<PathBuf>,
    pub locale: String,
}

pub struct LinuxAdapter {
    pub paths: LinuxPaths,
}

impl LinuxPaths {
    pub fn from_environment() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok())
    }
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let absolute = |key| lookup(key).filter(|value| Path::new(value).is_absolute());
        let home = absolute("HOME").map(PathBuf::from);
        let user_data = absolute("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|p| p.join(".local/share")));
        let mut data_dirs: Vec<_> = user_data.into_iter().collect();
        data_dirs.extend(
            lookup("XDG_DATA_DIRS")
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| "/usr/local/share:/usr/share".into())
                .split(':')
                .map(PathBuf::from)
                .filter(|p| p.is_absolute()),
        );
        let executable_dirs = lookup("PATH")
            .unwrap_or_default()
            .split(':')
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .collect();
        let mut icon_dirs: Vec<_> = home.map(|p| p.join(".icons")).into_iter().collect();
        icon_dirs.extend(data_dirs.iter().map(|p| p.join("icons")));
        icon_dirs.extend(data_dirs.iter().map(|p| p.join("pixmaps")));
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|key| lookup(key).filter(|v| !v.is_empty()))
            .unwrap_or_else(|| "C".into());
        Self {
            data_dirs,
            executable_dirs,
            icon_dirs,
            locale,
        }
    }
}

pub struct ProcSource {
    pub root: PathBuf,
}
impl super::ProcessSource for ProcSource {
    fn snapshot(&self) -> ProcessSnapshot {
        let mut snapshot = ProcessSnapshot::default();
        let Ok(entries) = fs::read_dir(&self.root) else {
            snapshot.incomplete = true;
            return snapshot;
        };
        for entry in entries {
            let Ok(entry) = entry else {
                snapshot.incomplete = true;
                continue;
            };
            if !entry
                .file_name()
                .to_string_lossy()
                .bytes()
                .all(|ch| ch.is_ascii_digit())
            {
                continue;
            }
            let root = entry.path();
            let name = fs::read_to_string(root.join("comm"));
            let executable = fs::read_link(root.join("exe")).ok();
            if name.is_err() || executable.is_none() {
                snapshot.incomplete = true;
            }
            snapshot.processes.push(super::ProcessInfo {
                executable,
                name: name.unwrap_or_default().trim_end_matches('\n').to_owned(),
            });
        }
        snapshot
    }
}

pub struct CommandSpawner;
impl CommandSpawner {
    pub fn command(launch: &ResolvedLaunch) -> std::process::Command {
        let mut command = std::process::Command::new(&launch.executable);
        command
            .args(&launch.arguments)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if let Some(path) = &launch.working_directory {
            command.current_dir(path);
        }
        command
    }
}
impl super::ProcessSpawner for CommandSpawner {
    fn spawn(&self, launch: &ResolvedLaunch) -> Result<(), ErrorCategory> {
        let mut child = Self::command(launch).spawn().map_err(io_category)?;
        // Reap without waiting for a long-lived GUI application in the caller.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

impl PlatformAdapter for LinuxAdapter {
    fn discover(&self) -> Result<Vec<InstalledApp>, ErrorCategory> {
        let mut apps = Vec::new();
        for (id, path) in self.desktop_files() {
            let Ok(entry) = self.read_entry(&path) else {
                continue;
            };
            let Ok(launch) = self.resolve_entry(&id, &path, &entry, &[]) else {
                continue;
            };
            apps.push(InstalledApp {
                id: id.clone(),
                name: entry.name,
                icon: entry.icon,
                launch_ref: LaunchRef::DesktopEntry {
                    desktop_id: id,
                    executable_hint: (!launch.uncertain_process_identity)
                        .then(|| launch.executable.to_string_lossy().into_owned()),
                },
            });
        }
        Ok(apps)
    }
    fn resolve(&self, target: &AppTarget) -> Result<ResolvedLaunch, ErrorCategory> {
        if target.arguments.len() > 64
            || target
                .arguments
                .iter()
                .any(|v| v.len() > 4096 || v.contains('\0'))
        {
            return Err(ErrorCategory::InvalidConfiguration);
        }
        match target.linux.as_ref().ok_or(ErrorCategory::NotFound)? {
            LaunchRef::Executable { path } => {
                if !Path::new(path).is_absolute() {
                    return Err(ErrorCategory::InvalidConfiguration);
                }
                let executable = self.executable(path)?;
                Ok(ResolvedLaunch {
                    uncertain_process_identity: wrapper(&executable),
                    executable,
                    arguments: target.arguments.clone(),
                    working_directory: None,
                })
            }
            LaunchRef::DesktopEntry { desktop_id, .. } => {
                let files = self.desktop_files();
                let (id, path) = files
                    .get_key_value(desktop_id)
                    .or_else(|| {
                        let aliases: &[&str] = match desktop_id.as_str() {
                            "antigravity.desktop" => {
                                &["Antigravity_IDE.desktop", "antigravity-ide.desktop"]
                            }
                            "brave-browser.desktop" => &["com.brave.Browser.desktop"],
                            _ => &[],
                        };
                        aliases.iter().find_map(|id| files.get_key_value(*id))
                    })
                    .ok_or(ErrorCategory::NotFound)?;
                self.resolve_entry(id, path, &self.read_entry(path)?, &target.arguments)
            }
            _ => Err(ErrorCategory::Unsupported),
        }
    }
    fn icon(&self, app: &InstalledApp, size: u32, scale: u32) -> AppIcon {
        let Some(name) = app.icon.as_ref().filter(|v| !v.is_empty()) else {
            return AppIcon::Fallback;
        };
        let path = Path::new(name);
        if path.is_absolute() {
            return if readable_file(path) {
                AppIcon::File(path.into())
            } else {
                AppIcon::Fallback
            };
        }
        if name.contains(['/', '\\']) || name.starts_with('.') {
            return AppIcon::Fallback;
        }
        let pixels = size.clamp(1, 512).saturating_mul(scale.clamp(1, 4));
        let mut sizes = vec![pixels, 16, 22, 24, 32, 48, 64, 96, 128, 256, 512];
        sizes.sort_by_key(|value| (value.abs_diff(pixels), *value < pixels));
        sizes.dedup();
        let names = if path.extension().is_some() {
            vec![name.clone()]
        } else {
            ["png", "svg", "xpm"]
                .map(|ext| format!("{name}.{ext}"))
                .into()
        };
        for root in &self.paths.icon_dirs {
            for theme in ["hicolor", "Adwaita", "breeze", "breeze-dark"] {
                let mut folders = vec![format!("{theme}/{}x{}@{}/apps", size, size, scale)];
                folders.extend(
                    sizes
                        .iter()
                        .map(|size| format!("{theme}/{size}x{size}/apps")),
                );
                folders.push(format!("{theme}/scalable/apps"));
                for folder in folders {
                    for name in &names {
                        let path = root.join(&folder).join(name);
                        if readable_file(&path) {
                            return AppIcon::File(path);
                        }
                    }
                }
            }
            for name in &names {
                let path = root.join(name);
                if readable_file(&path) {
                    return AppIcon::File(path);
                }
            }
        }
        AppIcon::Fallback
    }
    fn running(&self, target: &AppTarget, snapshot: &ProcessSnapshot) -> RunningState {
        let Ok(launch) = self.resolve(target) else {
            return RunningState::Unknown;
        };
        let mut paths = (!launch.uncertain_process_identity)
            .then(|| launch.executable.clone())
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut uncertain = snapshot.incomplete || launch.uncertain_process_identity;
        let mut names = Vec::new();
        for matcher in &target.process_matchers {
            match matcher {
                ProcessMatcher::Executable {
                    platform: Platform::Linux,
                    path,
                } => match fs::canonicalize(path) {
                    Ok(path) => {
                        paths.insert(path);
                    }
                    Err(_) => uncertain = true,
                },
                ProcessMatcher::Name {
                    platform: Platform::Linux,
                    name,
                } => names.push(name.as_str()),
                _ => {}
            }
        }
        let base = launch
            .executable
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("");
        let brave = matches!(target.linux.as_ref(), Some(LaunchRef::DesktopEntry { desktop_id, .. }) if desktop_id == "brave-browser.desktop")
            || matches!(base, "brave" | "brave-browser" | "brave-browser-stable");
        let antigravity = matches!(target.linux.as_ref(), Some(LaunchRef::DesktopEntry { desktop_id, .. }) if desktop_id == "antigravity.desktop")
            || base == "antigravity";
        let mut alias_paths = BTreeSet::new();
        for process in &snapshot.processes {
            let Some(path) = process
                .executable
                .as_ref()
                .and_then(|p| fs::canonicalize(p).ok())
            else {
                uncertain = true;
                continue;
            };
            if paths.contains(&path) {
                return RunningState::Running;
            }
            let process_base = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
            if ((brave
                && matches!(
                    process_base,
                    "brave" | "brave-browser" | "brave-browser-stable"
                ))
                || (antigravity && process_base == "antigravity"))
                && !wrapper(&path)
            {
                alias_paths.insert(path.clone());
            }
            if names.contains(&process.name.as_str()) || process_base == base {
                uncertain = true;
            }
        }
        if alias_paths.len() == 1 {
            return RunningState::Running;
        }
        if alias_paths.len() > 1 {
            return RunningState::Unknown;
        }
        if uncertain {
            RunningState::Unknown
        } else {
            RunningState::NotRunning
        }
    }
}

impl LinuxAdapter {
    fn desktop_files(&self) -> BTreeMap<String, PathBuf> {
        let mut result = BTreeMap::new();
        for dir in &self.paths.data_dirs {
            let root = dir.join("applications");
            let mut files = Vec::new();
            collect_files(&root, 0, &mut files);
            files.sort();
            for path in files {
                if path.extension().is_some_and(|ext| ext == "desktop")
                    && let Ok(relative) = path.strip_prefix(&root)
                {
                    let id = relative.to_string_lossy().replace('/', "-");
                    if !id.starts_with('-') {
                        result.entry(id).or_insert(path);
                    }
                }
            }
        }
        result
    }
    fn read_entry(&self, path: &Path) -> Result<DesktopEntry, ErrorCategory> {
        let mut text = String::new();
        fs::File::open(path)
            .map_err(io_category)?
            .take(1024 * 1024 + 1)
            .read_to_string(&mut text)
            .map_err(io_category)?;
        if text.len() > 1024 * 1024 {
            return Err(ErrorCategory::Unsupported);
        }
        let entry = DesktopEntry::parse(&text, &self.paths.locale)?;
        if let Some(try_exec) = &entry.try_exec {
            self.executable(try_exec)?;
        }
        Ok(entry)
    }
    fn executable(&self, value: &str) -> Result<PathBuf, ErrorCategory> {
        let path = Path::new(value);
        let candidates = if path.is_absolute() {
            vec![path.to_owned()]
        } else if path.components().count() == 1 {
            self.paths
                .executable_dirs
                .iter()
                .filter(|p| p.is_absolute())
                .map(|dir| dir.join(path))
                .collect()
        } else {
            return Err(ErrorCategory::Unsupported);
        };
        let mut error = ErrorCategory::NotFound;
        for candidate in candidates {
            let canonical = match fs::canonicalize(&candidate) {
                Ok(path) => path,
                Err(e) => {
                    error = io_category(e);
                    continue;
                }
            };
            let name = canonical.file_name().and_then(|v| v.to_str()).unwrap_or("");
            let original_name = candidate.file_name().and_then(|v| v.to_str()).unwrap_or("");
            if [name, original_name].iter().any(|name| {
                matches!(
                    *name,
                    "sh"
                        | "bash"
                        | "dash"
                        | "zsh"
                        | "fish"
                        | "ksh"
                        | "csh"
                        | "tcsh"
                        // `env` can select an interpreter or alter its environment. Parsing
                        // its mini-language is deliberately out of scope for a direct launcher.
                        | "env"
                )
            }) {
                return Err(ErrorCategory::Unsupported);
            }
            let metadata = fs::metadata(&canonical).map_err(io_category)?;
            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                return Ok(canonical);
            }
            error = ErrorCategory::PermissionDenied;
        }
        Err(error)
    }
    fn resolve_entry(
        &self,
        id: &str,
        path: &Path,
        entry: &DesktopEntry,
        custom: &[String],
    ) -> Result<ResolvedLaunch, ErrorCategory> {
        let working_directory = entry.working_directory.as_ref().map(PathBuf::from);
        if working_directory
            .as_ref()
            .is_some_and(|path| !path.is_absolute() || !path.is_dir())
        {
            return Err(ErrorCategory::NotFound);
        }
        if let Some(command) = entry.command(path)? {
            match self.executable(&command[0]) {
                Ok(executable) => {
                    let mut arguments = command[1..].to_vec();
                    arguments.extend_from_slice(custom);
                    return Ok(ResolvedLaunch {
                        uncertain_process_identity: wrapper(&executable),
                        executable,
                        arguments,
                        working_directory,
                    });
                }
                Err(ErrorCategory::NotFound) if entry.dbus => {}
                Err(error) => return Err(error),
            }
        }
        if !entry.dbus || !custom.is_empty() {
            return Err(ErrorCategory::Unsupported);
        }
        Ok(ResolvedLaunch {
            executable: self.executable("gtk-launch")?,
            arguments: vec![id.into()],
            working_directory: None,
            uncertain_process_identity: true,
        })
    }
}

fn readable_file(path: &Path) -> bool {
    path.is_file() && fs::File::open(path).is_ok()
}
fn wrapper(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|v| v.to_str())
        .is_some_and(|v| matches!(v, "env" | "flatpak" | "snap" | "gtk-launch"))
    {
        return true;
    }
    let mut header = [0; 2];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_err()
        || header == *b"#!"
}
fn collect_files(root: &Path, depth: usize, files: &mut Vec<PathBuf>) {
    if depth > 12 {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            collect_files(&path, depth + 1, files);
        } else {
            files.push(path);
        }
    }
}
fn io_category(error: std::io::Error) -> ErrorCategory {
    match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCategory::NotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCategory::PermissionDenied,
        _ => ErrorCategory::Io,
    }
}
