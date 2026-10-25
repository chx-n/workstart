use workstart_lib::domain::RunningState;
use workstart_lib::windows::*;

fn record(source: Source, name: &str, executable: &str) -> Record {
    Record {
        source,
        name: name.into(),
        executable: Some(executable.into()),
        shortcut: None,
    }
}

#[test]
fn app_paths_precede_uninstall_and_shortcuts_with_user_precedence() {
    let records = vec![
        record(Source::MachineUninstall, "Brave", r"C:\Other\brave.exe"),
        record(
            Source::MachineAppPaths,
            "brave.exe",
            r"C:\Machine\brave.exe",
        ),
        record(Source::UserAppPaths, "brave.exe", r"C:\User\brave.exe"),
    ];
    let apps = discover(records, &[], |_| None, |p| Some(p.into()));
    assert_eq!(
        resolve_default(&apps, "Brave").unwrap().executable,
        r"C:\User\brave.exe"
    );
    assert_eq!(apps.len(), 1);
}

#[test]
fn shortcuts_resolve_only_file_targets_and_dedupe_canonical_paths() {
    let mut records = vec![record(
        Source::UserUninstall,
        "Editor",
        r"C:\Apps\editor.exe",
    )];
    for name in ["Editor duplicate", "Store app", "Broken"] {
        records.push(Record {
            source: Source::UserShortcut,
            name: name.into(),
            executable: None,
            shortcut: Some(name.into()),
        });
    }
    let apps = discover(
        records,
        &[],
        |p| match p {
            "Editor duplicate" => Some(r"c:/apps/EDITOR.exe".into()),
            "Store app" => Some("shell:AppsFolder\\package!app".into()),
            _ => None,
        },
        |p| Some(p.into()),
    );
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "Editor");
}

#[test]
fn known_locations_are_fallbacks_and_missing_defaults_remain_unresolved() {
    let locations = known_locations(
        Some(r"C:\Users\me\AppData\Local"),
        Some(r"C:\Program Files"),
        None,
    );
    let apps = discover(
        vec![],
        &locations,
        |_| None,
        |p| p.ends_with("Antigravity.exe").then(|| p.into()),
    );
    assert!(resolve_default(&apps, "Antigravity").is_some());
    assert!(resolve_default(&apps, "Brave").is_none());
}

#[test]
fn missing_defaults_are_exposed_for_the_executable_picker() {
    let apps = discover(
        vec![record(
            Source::UserAppPaths,
            "Brave Browser",
            r"C:\Apps\brave.exe",
        )],
        &[],
        |_| None,
        |path| Some(path.into()),
    );

    assert_eq!(
        unresolved_defaults(&apps),
        vec![UnresolvedApplication {
            name: "Antigravity".into(),
        }]
    );
}

#[test]
fn paths_and_display_strings_cannot_supply_commands() {
    let records = [
        "brave.exe --flag",
        r"C:\Apps\app.exe --flag",
        "shell:AppsFolder\\x",
        r"C:\Apps\..\app.exe",
        r"C:\Apps\app.cmd",
        r"C:\Apps\app.exe:evil.exe",
    ]
    .map(|p| record(Source::UserAppPaths, "Danger", p));
    assert!(discover(records.to_vec(), &[], |_| None, |p| Some(p.into())).is_empty());
    assert!(prepare_launch("Brave", &[]).is_err());
}

#[test]
fn direct_launch_preserves_argument_boundaries() {
    let args = vec![
        "a b".into(),
        "$(whoami)".into(),
        "& exit".into(),
        "\"quoted\"".into(),
        "".into(),
    ];
    let launch = prepare_launch(r"C:\Apps\editor.exe", &args).unwrap();
    assert_eq!(launch.arguments(), args);
    assert_eq!(launch.executable(), r"C:\Apps\editor.exe");
    assert!(prepare_launch(r"C:\Apps\editor.exe", &["bad\0arg".into()]).is_err());
}

#[test]
fn running_uses_exact_paths_then_known_aliases_and_unknown_on_ambiguity() {
    let process = |path: Option<&str>, name: &str| Process {
        canonical_path: path.map(str::to_owned),
        name: name.into(),
    };
    assert_eq!(
        running_state(
            r"C:\Apps\editor.exe",
            &[process(Some(r"c:/apps/EDITOR.exe"), "anything")],
            true
        ),
        RunningState::Running
    );
    assert_eq!(
        running_state(r"C:\Apps\brave.exe", &[process(None, "brave.exe")], true),
        RunningState::Running
    );
    assert_eq!(
        running_state(r"C:\Apps\editor.exe", &[process(None, "editor.exe")], true),
        RunningState::Unknown
    );
    assert_eq!(
        running_state(r"C:\Apps\editor.exe", &[], false),
        RunningState::Unknown
    );
    assert_eq!(
        running_state(r"C:\Apps\editor.exe", &[], true),
        RunningState::NotRunning
    );
    assert_eq!(
        running_state(
            r"C:\Apps\brave.exe",
            &[process(Some(r"D:\Other\brave.exe"), "brave.exe")],
            true
        ),
        RunningState::NotRunning
    );
}

#[test]
fn icons_validate_pixel_bounds_and_fall_back_explicitly() {
    assert_eq!(lookup_icon(r"C:\Apps\editor.exe", |_| None), Icon::Generic);
    assert_eq!(
        lookup_icon(r"C:\Apps\editor.exe", |_| Some(RgbaIcon {
            width: 32,
            height: 32,
            pixels: vec![0; 2]
        })),
        Icon::Generic
    );
    assert!(matches!(
        lookup_icon(r"C:\Apps\editor.exe", |_| Some(RgbaIcon {
            width: 1,
            height: 1,
            pixels: vec![1, 2, 3, 255]
        })),
        Icon::Rgba(_)
    ));
}

#[test]
fn uninstall_records_use_exe_icons_or_known_install_dirs_never_uninstall_commands() {
    assert_eq!(
        uninstall_executable("Editor", Some(r#""C:\Apps\Editor.exe",-12"#), None),
        Some(r"C:\Apps\Editor.exe".into())
    );
    assert_eq!(
        uninstall_executable("Brave", Some(r"C:\Apps\icon.dll,0"), Some(r"D:\Brave")),
        Some(r"D:\Brave\Application\brave.exe".into())
    );
    assert_eq!(
        uninstall_executable("Unknown", None, Some(r"C:\Apps")),
        None
    );
    assert_eq!(
        uninstall_executable("Editor", Some(r"C:\Apps\editor.exe --remove"), None),
        None
    );
}

// Compile native entry points on Windows without touching registry, processes or
// launching an application. Runtime smoke tests belong to Task 9.
#[cfg(windows)]
#[test]
fn native_entry_points_have_the_adapter_contract() {
    let _: fn() -> Vec<Application> = native::discover_installed;
    let _: fn(&str) -> Icon = native::application_icon;
    let _: fn(&str) -> RunningState = native::is_running;
    let _: fn(&DirectLaunch) -> Result<u32, workstart_lib::domain::ErrorCategory> = native::launch;
}
