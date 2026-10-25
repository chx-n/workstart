use crate::domain::{ErrorCategory, WorkflowConfig};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadResult {
    pub workflow: WorkflowConfig,
    pub status: LoadStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoadStatus {
    Loaded,
    Missing,
    Recovered {
        backup_file: String,
        reason: RecoveryReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReason {
    Corrupt,
    UnsupportedVersion,
    InvalidConfiguration,
}

#[derive(Debug, Serialize)]
pub struct ConfigError {
    pub category: ErrorCategory,
    pub message: String,
}

/// Only the backend constructs this store. IPC never accepts a config path.
pub struct ConfigStore {
    directory: PathBuf,
}

/// Serialize reads (including recovery) and saves across both Tauri windows.
pub struct WorkflowState(std::sync::Mutex<ConfigStore>);

impl WorkflowState {
    pub fn new(directory: PathBuf) -> Self {
        Self(std::sync::Mutex::new(ConfigStore::new(directory)))
    }

    pub fn get_workflow(&self) -> Result<LoadResult, ConfigError> {
        self.0.lock().map_err(|_| state_error())?.load()
    }

    pub fn save_workflow(&self, workflow: &WorkflowConfig) -> Result<(), ConfigError> {
        self.0.lock().map_err(|_| state_error())?.save(workflow)
    }
}

fn state_error() -> ConfigError {
    ConfigError {
        category: ErrorCategory::Io,
        message: "Workflow state is unavailable. Restart Workstart.".into(),
    }
}

#[tauri::command]
pub fn get_workflow(state: tauri::State<'_, WorkflowState>) -> Result<LoadResult, ConfigError> {
    state.get_workflow()
}

#[tauri::command]
pub fn save_workflow(
    state: tauri::State<'_, WorkflowState>,
    workflow: WorkflowConfig,
) -> Result<(), ConfigError> {
    state.save_workflow(&workflow)
}

impl ConfigStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
    pub fn load(&self) -> Result<LoadResult, ConfigError> {
        let path = self.directory.join("workflow.json");
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(LoadResult {
                    workflow: WorkflowConfig::default(),
                    status: LoadStatus::Missing,
                });
            }
            Err(error) => return Err(io_error(error)),
        };
        // Arguments can expand sixfold when JSON escapes control characters.
        // This remains above the largest valid serialized workflow while still
        // bounding memory use before deserialization.
        const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
        let mut contents = Vec::new();
        file.take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut contents)
            .map_err(io_error)?;
        let parsed = if contents.len() as u64 > MAX_FILE_BYTES {
            Err(RecoveryReason::InvalidConfiguration)
        } else {
            parse_workflow(&contents)
        };
        match parsed {
            Ok(workflow) => Ok(LoadResult {
                workflow,
                status: LoadStatus::Loaded,
            }),
            Err(reason) => {
                let backup = unused_recovery_path(&self.directory).map_err(io_error)?;
                fs::rename(&path, &backup).map_err(io_error)?;
                let backup_file = backup
                    .file_name()
                    .expect("backup has a filename")
                    .to_string_lossy()
                    .into_owned();
                Ok(LoadResult {
                    workflow: WorkflowConfig::default(),
                    status: LoadStatus::Recovered {
                        backup_file,
                        reason,
                    },
                })
            }
        }
    }
    pub fn save(&self, workflow: &WorkflowConfig) -> Result<(), ConfigError> {
        workflow.validate().map_err(|error| ConfigError {
            category: ErrorCategory::InvalidConfiguration,
            message: error.to_string(),
        })?;
        let contents = serde_json::to_vec_pretty(workflow).map_err(|_| ConfigError {
            category: ErrorCategory::InvalidConfiguration,
            message: "The workflow could not be serialized.".into(),
        })?;
        fs::create_dir_all(&self.directory).map_err(io_error)?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".workflow-")
            .tempfile_in(&self.directory)
            .map_err(io_error)?;
        temporary.write_all(&contents).map_err(io_error)?;
        temporary.write_all(b"\n").map_err(io_error)?;
        temporary.as_file().sync_all().map_err(io_error)?;
        temporary
            .persist(self.directory.join("workflow.json"))
            .map_err(|error| io_error(error.error))?;
        Ok(())
    }
}

/// Choose an unpredictable same-directory destination, then release the
/// temporary file before rename. Windows cannot rename over a destination
/// that is still open, while the random name makes a collision negligible.
fn unused_recovery_path(directory: &Path) -> io::Result<PathBuf> {
    let reservation = tempfile::Builder::new()
        .prefix("workflow.recovered-")
        .suffix(".json")
        .tempfile_in(directory)?;
    let path = reservation.path().to_path_buf();
    reservation.close()?;
    Ok(path)
}

fn parse_workflow(contents: &[u8]) -> Result<WorkflowConfig, RecoveryReason> {
    let value: serde_json::Value =
        serde_json::from_slice(contents).map_err(|_| RecoveryReason::Corrupt)?;
    let version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or(RecoveryReason::InvalidConfiguration)?;
    if version != u64::from(crate::domain::SCHEMA_VERSION) {
        return Err(RecoveryReason::UnsupportedVersion);
    }
    let workflow: WorkflowConfig =
        serde_json::from_value(value).map_err(|_| RecoveryReason::InvalidConfiguration)?;
    workflow
        .validate()
        .map_err(|_| RecoveryReason::InvalidConfiguration)?;
    Ok(workflow)
}

fn io_error(error: io::Error) -> ConfigError {
    let category = if error.kind() == io::ErrorKind::PermissionDenied {
        ErrorCategory::PermissionDenied
    } else {
        ErrorCategory::Io
    };
    // Avoid exposing local paths or the config contents in frontend errors.
    ConfigError {
        category,
        message: "The local workflow file could not be accessed.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::unused_recovery_path;

    #[test]
    fn recovery_rename_destination_is_released_before_use() {
        let directory = tempfile::tempdir().unwrap();
        let path = unused_recovery_path(directory.path()).unwrap();

        assert_eq!(path.parent(), Some(directory.path()));
        assert!(!path.exists());
    }
}
