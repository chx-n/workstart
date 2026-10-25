use tauri::Manager;

pub mod config;
pub mod domain;
pub mod orchestration;
pub mod platform;
pub mod windows;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveredAppView {
    id: String,
    name: String,
    launch_ref: config::LaunchRefView,
    process_matchers: Vec<config::ProcessMatcherView>,
}

#[tauri::command]
fn discover_installed_apps() -> Result<Vec<DiscoveredAppView>, config::ConfigError> {
    #[cfg(target_os = "linux")]
    {
        use platform::PlatformAdapter;
        let adapter = platform::linux::LinuxAdapter {
            paths: platform::linux::LinuxPaths::from_environment(),
        };
        return adapter
            .discover()
            .map(|apps| {
                apps.into_iter()
                    .map(|app| DiscoveredAppView {
                        id: app.id,
                        name: app.name,
                        launch_ref: app.launch_ref.into(),
                        process_matchers: Vec::new(),
                    })
                    .collect()
            })
            .map_err(|category| config::ConfigError {
                category,
                message: "Installed apps could not be discovered.".into(),
            });
    }
    #[cfg(target_os = "windows")]
    {
        return Ok(windows::native::discover_installed()
            .into_iter()
            .map(|app| DiscoveredAppView {
                id: app.executable.clone(),
                name: app.name,
                launch_ref: config::LaunchRefView::Executable {
                    value: app.executable,
                },
                process_matchers: Vec::new(),
            })
            .collect());
    }
    #[allow(unreachable_code)]
    Ok(Vec::new())
}

#[tauri::command]
fn get_app_icon(_app_id: String) -> Result<Option<String>, config::ConfigError> {
    Ok(None)
}

#[tauri::command]
fn pick_executable() -> Result<Option<serde_json::Value>, config::ConfigError> {
    Err(config::ConfigError {
        category: domain::ErrorCategory::Unsupported,
        message: "Executable picker is unavailable in this build.".into(),
    })
}

#[tauri::command]
fn export_logs() -> Result<(), config::ConfigError> {
    Err(config::ConfigError {
        category: domain::ErrorCategory::Unsupported,
        message: "Choose a diagnostics destination from a packaged build.".into(),
    })
}

#[cfg(target_os = "linux")]
struct LinuxRuntime {
    adapter: platform::linux::LinuxAdapter,
    snapshot: platform::ProcessSnapshot,
}

#[cfg(target_os = "linux")]
impl LinuxRuntime {
    fn new() -> Self {
        use platform::ProcessSource;

        let adapter = platform::linux::LinuxAdapter {
            paths: platform::linux::LinuxPaths::from_environment(),
        };
        let snapshot = platform::linux::ProcSource {
            root: "/proc".into(),
        }
        .snapshot();
        Self { adapter, snapshot }
    }
}

#[cfg(target_os = "linux")]
impl orchestration::TargetRuntime for LinuxRuntime {
    fn running(&self, target: &domain::AppTarget) -> domain::RunningState {
        use platform::PlatformAdapter;
        self.adapter.running(target, &self.snapshot)
    }

    fn launch(&self, target: &domain::AppTarget) -> Result<(), domain::ErrorCategory> {
        use platform::PlatformAdapter;
        self.adapter
            .launch(target, &platform::linux::CommandSpawner)
    }
}

#[cfg(target_os = "windows")]
struct WindowsRuntime {
    applications: Vec<windows::Application>,
}

#[cfg(target_os = "windows")]
impl WindowsRuntime {
    fn new() -> Self {
        Self {
            applications: windows::native::discover_installed(),
        }
    }

    fn plan(
        &self,
        target: &domain::AppTarget,
    ) -> Result<windows::DirectLaunch, domain::ErrorCategory> {
        let reference = target
            .windows
            .as_ref()
            .ok_or(domain::ErrorCategory::NotFound)?;
        let executable = match reference {
            domain::LaunchRef::Executable { path } => path.as_str(),
            domain::LaunchRef::WindowsApp { app_name } => {
                windows::resolve_default(&self.applications, app_name)
                    .ok_or(domain::ErrorCategory::NotFound)?
                    .executable
                    .as_str()
            }
            domain::LaunchRef::DesktopEntry { .. } => {
                return Err(domain::ErrorCategory::Unsupported);
            }
        };
        windows::prepare_launch(executable, &target.arguments)
    }
}

#[cfg(target_os = "windows")]
impl orchestration::TargetRuntime for WindowsRuntime {
    fn running(&self, target: &domain::AppTarget) -> domain::RunningState {
        self.plan(target)
            .map(|plan| windows::native::is_running(plan.executable()))
            .unwrap_or(domain::RunningState::Unknown)
    }

    fn launch(&self, target: &domain::AppTarget) -> Result<(), domain::ErrorCategory> {
        windows::native::launch(&self.plan(target)?).map(|_| ())
    }
}

#[tauri::command]
fn launch_targets(
    state: tauri::State<'_, config::WorkflowState>,
    log: tauri::State<'_, orchestration::LaunchLogState>,
    target_ids: Vec<String>,
) -> Result<Vec<orchestration::TargetLaunchResult>, config::ConfigError> {
    let workflow = state.load_for_launch()?;

    #[cfg(target_os = "linux")]
    let results =
        orchestration::launch_from_stored_ids(&workflow, &target_ids, &LinuxRuntime::new(), &*log);

    #[cfg(target_os = "windows")]
    let results = orchestration::launch_from_stored_ids(
        &workflow,
        &target_ids,
        &WindowsRuntime::new(),
        &*log,
    );

    Ok(results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let config_directory = app.path().app_config_dir()?;
            app.manage(config::WorkflowState::new(config_directory.clone()));
            app.manage(orchestration::LaunchLogState::new(
                config_directory.join("logs"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_workflow,
            config::save_workflow,
            launch_targets,
            discover_installed_apps,
            get_app_icon,
            pick_executable,
            export_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running Workstart");
}
