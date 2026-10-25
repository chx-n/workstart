use tauri::Manager;

pub mod config;
pub mod domain;
pub mod windows;

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
            app.manage(config::WorkflowState::new(app.path().app_config_dir()?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_workflow,
            config::save_workflow
        ])
        .run(tauri::generate_context!())
        .expect("error while running Workstart");
}
