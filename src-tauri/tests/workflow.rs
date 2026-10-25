use workstart_lib::domain::{LaunchRef, Platform, ProcessMatcher, WorkflowConfig};

#[test]
fn runtime_results_are_serializable_without_unstructured_process_errors() {
    use workstart_lib::domain::{ErrorCategory, LaunchOutcome, RunningState};
    let state = RunningState::Unknown;
    assert_eq!(serde_json::to_value(state).unwrap(), "unknown");
    for outcome in [
        LaunchOutcome::AlreadyOpen,
        LaunchOutcome::Started,
        LaunchOutcome::Failed {
            category: ErrorCategory::NotFound,
        },
    ] {
        let json = serde_json::to_string(&outcome).unwrap();
        assert_eq!(
            serde_json::from_str::<LaunchOutcome>(&json).unwrap(),
            outcome
        );
    }
}

#[test]
fn defaults_are_stable_enabled_antigravity_and_brave_targets() {
    let config = WorkflowConfig::default();
    assert_eq!(config.schema_version, 1);
    assert_eq!(config.targets.len(), 2);
    assert_eq!(config.targets[0].name, "Antigravity IDE");
    assert_eq!(config.targets[1].name, "Brave");
    assert!(config.targets.iter().all(|target| target.enabled));
    assert_ne!(config.targets[0].id, config.targets[1].id);
    assert_eq!(config, WorkflowConfig::default());
    assert!(
        matches!(&config.targets[0].linux, Some(LaunchRef::DesktopEntry { desktop_id, executable_hint: Some(_) }) if desktop_id == "antigravity.desktop")
    );
    assert!(
        matches!(&config.targets[1].linux, Some(LaunchRef::DesktopEntry { desktop_id, executable_hint: Some(_) }) if desktop_id == "brave-browser.desktop")
    );
    config.validate().unwrap();
}

#[test]
fn workflow_round_trips_without_using_host_path_semantics() {
    let mut config = WorkflowConfig::default();
    config.targets[0].windows = Some(LaunchRef::Executable {
        path: r"C:\Program Files\Antigravity\Antigravity.exe".into(),
    });
    config.targets[0].arguments = vec!["--new-window".into(), "a space;$(literal)".into()];
    config.targets[0]
        .process_matchers
        .push(ProcessMatcher::Executable {
            platform: Platform::Windows,
            path: r"C:\Program Files\Antigravity\Antigravity.exe".into(),
        });
    config.validate().unwrap();
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(
        serde_json::from_str::<WorkflowConfig>(&json).unwrap(),
        config
    );
}

#[test]
fn validation_rejects_duplicate_ids_and_invalid_names() {
    let mut config = WorkflowConfig::default();
    config.targets[1].id = config.targets[0].id;
    assert!(config.validate().is_err());
    for name in ["", "   ", " Brave", "Brave ", "Brave\n"] {
        let mut config = WorkflowConfig::default();
        config.targets[0].name = name.into();
        assert!(config.validate().is_err(), "accepted {name:?}");
    }
}

#[test]
fn validation_rejects_invalid_references_and_platform_mismatches() {
    for path in [
        "",
        "brave",
        "./brave",
        "~/bin/brave",
        "C:\\Brave.exe",
        "/usr/bin/../brave",
        "/usr/bin/brave\0",
    ] {
        let mut config = WorkflowConfig::default();
        config.targets[0].linux = Some(LaunchRef::Executable { path: path.into() });
        assert!(config.validate().is_err(), "accepted Linux path {path:?}");
    }
    for path in [
        "C:brave.exe",
        "\\brave.exe",
        "/usr/bin/brave",
        "C:\\..\\brave.exe",
    ] {
        let mut config = WorkflowConfig::default();
        config.targets[0].windows = Some(LaunchRef::Executable { path: path.into() });
        assert!(config.validate().is_err(), "accepted Windows path {path:?}");
    }
    for id in [
        "",
        "../brave.desktop",
        "brave",
        "brave.desktop;sh",
        "-brave.desktop",
    ] {
        let mut config = WorkflowConfig::default();
        config.targets[0].linux = Some(LaunchRef::DesktopEntry {
            desktop_id: id.into(),
            executable_hint: None,
        });
        assert!(config.validate().is_err(), "accepted desktop ID {id:?}");
    }
    let mut config = WorkflowConfig::default();
    config.targets[0].windows = config.targets[0].linux.clone();
    assert!(config.validate().is_err());
    config.targets[0].linux = None;
    config.targets[0].windows = None;
    assert!(config.validate().is_err());
}

#[test]
fn validation_bounds_schema_targets_arguments_and_matchers() {
    let config = WorkflowConfig {
        schema_version: 2,
        ..WorkflowConfig::default()
    };
    assert!(config.validate().is_err());
    let mut config = WorkflowConfig::default();
    config.targets = vec![config.targets[0].clone(); 65];
    assert!(config.validate().is_err());
    for arguments in [
        vec!["a".into(); 65],
        vec!["a".repeat(4097)],
        vec!["a\0b".into()],
    ] {
        let mut config = WorkflowConfig::default();
        config.targets[0].arguments = arguments;
        assert!(config.validate().is_err());
    }
    let mut config = WorkflowConfig::default();
    config.targets[0].arguments = vec!["a".repeat(4096); 64];
    config.validate().unwrap();
    for matchers in [
        vec![
            ProcessMatcher::Name {
                platform: Platform::Linux,
                name: "brave".into()
            };
            17
        ],
        vec![ProcessMatcher::Name {
            platform: Platform::Linux,
            name: "b".repeat(256),
        }],
        vec![ProcessMatcher::Name {
            platform: Platform::Linux,
            name: "".into(),
        }],
    ] {
        let mut config = WorkflowConfig::default();
        config.targets[0].process_matchers = matchers;
        assert!(config.validate().is_err());
    }
    let mut config = WorkflowConfig::default();
    config.targets.clear();
    config.validate().unwrap();
}
