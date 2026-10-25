use std::path::PathBuf;

use crate::domain::{AppTarget, ErrorCategory, LaunchRef, RunningState};

#[cfg(target_os = "linux")]
pub mod linux;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub launch_ref: LaunchRef,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppIcon {
    File(PathBuf),
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLaunch {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub uncertain_process_identity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub executable: Option<PathBuf>,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessSnapshot {
    pub processes: Vec<ProcessInfo>,
    pub incomplete: bool,
}

pub trait ProcessSource {
    fn snapshot(&self) -> ProcessSnapshot;
}

pub trait ProcessSpawner {
    fn spawn(&self, launch: &ResolvedLaunch) -> Result<(), ErrorCategory>;
}

/// Backend-only boundary. IPC must resolve stored target IDs before using it.
pub trait PlatformAdapter {
    fn discover(&self) -> Result<Vec<InstalledApp>, ErrorCategory>;
    fn resolve(&self, target: &AppTarget) -> Result<ResolvedLaunch, ErrorCategory>;
    fn icon(&self, app: &InstalledApp, size: u32, scale: u32) -> AppIcon;
    fn running(&self, target: &AppTarget, snapshot: &ProcessSnapshot) -> RunningState;
    fn launch(
        &self,
        target: &AppTarget,
        spawner: &dyn ProcessSpawner,
    ) -> Result<(), ErrorCategory> {
        spawner.spawn(&self.resolve(target)?)
    }
}
