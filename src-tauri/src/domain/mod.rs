use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConfig {
    pub schema_version: u32,
    pub targets: Vec<AppTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppTarget {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub linux: Option<LaunchRef>,
    pub windows: Option<LaunchRef>,
    pub arguments: Vec<String>,
    pub process_matchers: Vec<ProcessMatcher>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LaunchRef {
    Executable {
        path: String,
    },
    DesktopEntry {
        desktop_id: String,
        executable_hint: Option<String>,
    },
    WindowsApp {
        app_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Linux,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProcessMatcher {
    Executable { platform: Platform, path: String },
    Name { platform: Platform, name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunningState {
    Running,
    NotRunning,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum LaunchOutcome {
    AlreadyOpen,
    Started,
    Failed { category: ErrorCategory },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    InvalidConfiguration,
    NotFound,
    PermissionDenied,
    Unsupported,
    Io,
    LaunchFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            targets: vec![
                AppTarget {
                    id: Uuid::from_u128(0x3a360d47_364b_423f_b574_9b7dcdd6a51a),
                    name: "Antigravity IDE".into(),
                    enabled: true,
                    linux: Some(LaunchRef::DesktopEntry {
                        desktop_id: "antigravity.desktop".into(),
                        executable_hint: Some("/usr/share/antigravity/antigravity".into()),
                    }),
                    windows: Some(LaunchRef::WindowsApp {
                        app_name: "Antigravity".into(),
                    }),
                    arguments: vec![],
                    process_matchers: vec![
                        ProcessMatcher::Name {
                            platform: Platform::Linux,
                            name: "antigravity".into(),
                        },
                        ProcessMatcher::Name {
                            platform: Platform::Windows,
                            name: "Antigravity.exe".into(),
                        },
                    ],
                },
                AppTarget {
                    id: Uuid::from_u128(0x613c1f4e_41b5_4f31_bda4_969f52891237),
                    name: "Brave".into(),
                    enabled: true,
                    linux: Some(LaunchRef::DesktopEntry {
                        desktop_id: "brave-browser.desktop".into(),
                        executable_hint: Some("/usr/bin/brave-browser".into()),
                    }),
                    windows: Some(LaunchRef::WindowsApp {
                        app_name: "Brave".into(),
                    }),
                    arguments: vec![],
                    process_matchers: vec![
                        ProcessMatcher::Name {
                            platform: Platform::Linux,
                            name: "brave".into(),
                        },
                        ProcessMatcher::Name {
                            platform: Platform::Windows,
                            name: "brave.exe".into(),
                        },
                    ],
                },
            ],
        }
    }
}

impl WorkflowConfig {
    pub fn validate(&self) -> Result<(), ValidationError> {
        check(
            self.schema_version == SCHEMA_VERSION,
            "schema_version",
            "Unsupported schema version.",
        )?;
        check(
            self.targets.len() <= 64,
            "targets",
            "At most 64 targets are allowed.",
        )?;
        let mut ids = std::collections::HashSet::new();
        for (index, target) in self.targets.iter().enumerate() {
            let field = format!("targets[{index}]");
            check(
                !target.id.is_nil() && ids.insert(target.id),
                &format!("{field}.id"),
                "Target IDs must be non-nil and unique.",
            )?;
            check(
                valid_name(&target.name, 128),
                &format!("{field}.name"),
                "Use a trimmed name of 1–128 bytes without control characters.",
            )?;
            check(
                target.linux.is_some() || target.windows.is_some(),
                &field,
                "A launch reference is required.",
            )?;
            for (platform, reference) in [
                (Platform::Linux, &target.linux),
                (Platform::Windows, &target.windows),
            ] {
                if let Some(reference) = reference {
                    check(
                        reference.is_valid_for(platform),
                        &format!("{field}.{platform:?}"),
                        "Invalid launch reference for this platform.",
                    )?;
                }
            }
            check(
                target.arguments.len() <= 64,
                &format!("{field}.arguments"),
                "At most 64 arguments are allowed.",
            )?;
            for argument in &target.arguments {
                check(
                    argument.len() <= 4096 && !argument.contains('\0'),
                    &format!("{field}.arguments"),
                    "Arguments must be at most 4096 bytes and contain no NUL.",
                )?;
            }
            check(
                target.process_matchers.len() <= 16,
                &format!("{field}.process_matchers"),
                "At most 16 process matchers are allowed.",
            )?;
            for matcher in &target.process_matchers {
                let valid = match matcher {
                    ProcessMatcher::Executable { platform, path } => valid_path(path, *platform),
                    ProcessMatcher::Name { name, .. } => {
                        valid_name(name, 255) && !name.contains(['/', '\\'])
                    }
                };
                check(
                    valid,
                    &format!("{field}.process_matchers"),
                    "Invalid process name or absolute executable path.",
                )?;
            }
        }
        Ok(())
    }
}

impl LaunchRef {
    fn is_valid_for(&self, platform: Platform) -> bool {
        match self {
            Self::Executable { path } => valid_path(path, platform),
            Self::DesktopEntry {
                desktop_id,
                executable_hint,
            } => {
                platform == Platform::Linux
                    && desktop_id.len() <= 255
                    && desktop_id.ends_with(".desktop")
                    && desktop_id.len() > ".desktop".len()
                    && desktop_id.as_bytes()[0].is_ascii_alphanumeric()
                    && desktop_id
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                    && executable_hint
                        .as_ref()
                        .is_none_or(|path| valid_path(path, platform))
            }
            Self::WindowsApp { app_name } => {
                platform == Platform::Windows && valid_name(app_name, 128)
            }
        }
    }
}

fn valid_name(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

/// Validate foreign paths lexically, never with host-specific Path::is_absolute
/// or existence checks. Discovery/launch resolves these before executing.
fn valid_path(path: &str, platform: Platform) -> bool {
    if !valid_name(path, 4096) {
        return false;
    }
    match platform {
        Platform::Linux => {
            path.starts_with('/')
                && !path.ends_with('/')
                && !path.split('/').any(|part| part == "." || part == "..")
        }
        Platform::Windows => {
            let normalized = path.replace('\\', "/");
            if normalized.ends_with('/') {
                return false;
            }
            let rest = if let Some(rest) = normalized.strip_prefix("//") {
                if rest.split('/').count() < 3 {
                    return false;
                }
                rest
            } else {
                let bytes = normalized.as_bytes();
                if bytes.len() < 4 || !bytes[0].is_ascii_alphabetic() || bytes[1..3] != *b":/" {
                    return false;
                }
                &normalized[3..]
            };
            rest.split('/').all(|part| {
                !part.is_empty()
                    && part != "."
                    && part != ".."
                    && !part.ends_with(['.', ' '])
                    && !part.contains(['<', '>', ':', '"', '|', '?', '*'])
            })
        }
    }
}

fn check(valid: bool, field: &str, message: &str) -> Result<(), ValidationError> {
    if valid {
        Ok(())
    } else {
        Err(ValidationError {
            field: field.into(),
            message: message.into(),
        })
    }
}
