//! Stored-target launch orchestration. This module never accepts executables,
//! command strings, or argument arrays from IPC.

use crate::domain::{AppTarget, ErrorCategory, RunningState, WorkflowConfig};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::{Mutex, mpsc},
    time::{SystemTime, UNIX_EPOCH},
};

const LOG_FILE: &str = "launch.log";
const LOG_MAX_BYTES: u64 = 1024 * 1024;
const LOG_FILE_COUNT: usize = 7;

/// The narrow platform boundary makes launch policy testable without starting
/// applications. Implementations must use only references stored in `target`.
pub trait TargetRuntime: Sync {
    fn running(&self, target: &AppTarget) -> RunningState;
    fn launch(&self, target: &AppTarget) -> Result<(), ErrorCategory>;
}

pub trait LaunchLog: Sync {
    fn record(&self, entry: &LogEntry) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetLaunchResult {
    pub target_id: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_category: Option<ErrorCategory>,
}

impl TargetLaunchResult {
    pub fn started(target_id: String) -> Self {
        Self {
            target_id,
            outcome: "started".into(),
            error_category: None,
        }
    }

    fn skipped(target_id: String) -> Self {
        Self {
            target_id,
            outcome: "skippedAlreadyRunning".into(),
            error_category: None,
        }
    }

    fn failed(target_id: String, error_category: ErrorCategory) -> Self {
        Self {
            target_id,
            outcome: "failed".into(),
            error_category: Some(error_category),
        }
    }
}

/// Structured and deliberately redacted. In particular, this does not contain
/// target names, executable paths, process data, or launch argument values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogEntry {
    event: &'static str,
    timestamp_unix_ms: u128,
    target_id: String,
    outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_category: Option<ErrorCategory>,
}

impl LogEntry {
    fn from_result(result: &TargetLaunchResult) -> Self {
        Self {
            event: "launch_result",
            timestamp_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            target_id: result.target_id.clone(),
            outcome: result.outcome.clone(),
            error_category: result.error_category,
        }
    }
}

/// A local JSON-lines sink. Files rotate before they exceed their bound, with
/// `launch.log` as the newest file and six numbered historical files.
pub struct RotatingLog {
    directory: PathBuf,
    max_bytes: u64,
    file_count: usize,
}

/// Shared application state serializes rotation across concurrent IPC calls.
pub struct LaunchLogState(Mutex<RotatingLog>);

impl LaunchLogState {
    pub fn new(directory: PathBuf) -> io::Result<Self> {
        Ok(Self(Mutex::new(RotatingLog::new(directory)?)))
    }
}

impl LaunchLog for LaunchLogState {
    fn record(&self, entry: &LogEntry) -> io::Result<()> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("launch log state is unavailable"))?
            .record(entry)
    }
}

impl RotatingLog {
    pub fn new(directory: PathBuf) -> io::Result<Self> {
        Self::with_limits(directory, LOG_MAX_BYTES, LOG_FILE_COUNT)
    }

    /// Public for fixture tests; production always uses `new`.
    pub fn with_limits(directory: PathBuf, max_bytes: u64, file_count: usize) -> io::Result<Self> {
        if max_bytes == 0 || file_count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "log limits must be positive",
            ));
        }
        fs::create_dir_all(&directory)?;
        Ok(Self {
            directory,
            max_bytes,
            file_count,
        })
    }

    pub fn record_result(&self, result: &TargetLaunchResult) -> io::Result<()> {
        self.record(&LogEntry::from_result(result))
    }

    fn current_path(&self) -> PathBuf {
        self.directory.join(LOG_FILE)
    }

    fn rotated_path(&self, index: usize) -> PathBuf {
        self.directory.join(format!("{LOG_FILE}.{index}"))
    }

    fn rotate(&self) -> io::Result<()> {
        if self.file_count == 1 {
            let path = self.current_path();
            if path.exists() {
                fs::remove_file(path)?;
            }
            return Ok(());
        }
        let oldest = self.rotated_path(self.file_count - 1);
        if oldest.exists() {
            fs::remove_file(oldest)?;
        }
        for index in (1..self.file_count - 1).rev() {
            let source = self.rotated_path(index);
            if source.exists() {
                fs::rename(source, self.rotated_path(index + 1))?;
            }
        }
        let current = self.current_path();
        if current.exists() {
            fs::rename(current, self.rotated_path(1))?;
        }
        Ok(())
    }

    fn regular_or_missing(path: &std::path::Path) -> io::Result<()> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(()),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "log path is not a regular file",
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl LaunchLog for RotatingLog {
    fn record(&self, entry: &LogEntry) -> io::Result<()> {
        let mut line = serde_json::to_vec(entry)
            .map_err(|_| io::Error::other("could not serialize launch log"))?;
        line.push(b'\n');
        if line.len() as u64 > self.max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "launch log record exceeds file limit",
            ));
        }
        let path = self.current_path();
        Self::regular_or_missing(&path)?;
        let existing = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if existing.saturating_add(line.len() as u64) > self.max_bytes {
            self.rotate()?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(&line)?;
        file.sync_data()
    }
}

/// Resolves each requested ID exclusively against the saved workflow. Every
/// requested target receives an independent outcome, in input order; this is
/// what makes retrying only failed IDs safe and natural for the UI.
pub fn launch_from_stored_ids(
    workflow: &WorkflowConfig,
    target_ids: &[String],
    runtime: &dyn TargetRuntime,
    log: &dyn LaunchLog,
) -> Vec<TargetLaunchResult> {
    let targets: HashMap<_, _> = workflow
        .targets
        .iter()
        .map(|target| (target.id.to_string(), target))
        .collect();
    let mut immediate = Vec::new();
    let mut launches = Vec::new();
    let mut seen = HashSet::new();

    for (index, target_id) in target_ids.iter().enumerate() {
        if !seen.insert(target_id) {
            immediate.push((
                index,
                TargetLaunchResult::failed(target_id.clone(), ErrorCategory::InvalidConfiguration),
            ));
            continue;
        }
        let Some(target) = targets.get(target_id) else {
            immediate.push((
                index,
                TargetLaunchResult::failed(target_id.clone(), ErrorCategory::NotFound),
            ));
            continue;
        };
        if !target.enabled {
            immediate.push((
                index,
                TargetLaunchResult::failed(target_id.clone(), ErrorCategory::InvalidConfiguration),
            ));
            continue;
        }
        launches.push((index, target_id.clone(), *target));
    }

    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        for (index, target_id, target) in launches {
            let sender = sender.clone();
            scope.spawn(move || {
                let result = match runtime.running(target) {
                    RunningState::Running => TargetLaunchResult::skipped(target_id),
                    RunningState::NotRunning | RunningState::Unknown => {
                        match runtime.launch(target) {
                            Ok(()) => TargetLaunchResult::started(target_id),
                            Err(error) => TargetLaunchResult::failed(target_id, error),
                        }
                    }
                };
                let _ = sender.send((index, result));
            });
        }
    });
    drop(sender);

    immediate.extend(receiver);
    immediate.sort_by_key(|(index, _)| *index);
    immediate
        .into_iter()
        .map(|(_, result)| {
            // Launch behavior is independent of local diagnostic storage.
            let _ = log.record(&LogEntry::from_result(&result));
            result
        })
        .collect()
}
