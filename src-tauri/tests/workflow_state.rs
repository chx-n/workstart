use workstart_lib::{
    config::WorkflowState,
    domain::{LaunchRef, Platform, ProcessMatcher, WorkflowConfig},
};

fn expected_launch_ref(linux: serde_json::Value, windows: serde_json::Value) -> serde_json::Value {
    #[cfg(target_os = "windows")]
    {
        let _ = linux;
        windows
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = windows;
        linux
    }
}

#[test]
fn managed_state_returns_the_exact_frontend_workflow_contract() {
    let dir = tempfile::tempdir().unwrap();
    let state = WorkflowState::new(dir.path().join("owned"));

    assert_eq!(
        serde_json::to_value(state.get_workflow().unwrap()).unwrap(),
        serde_json::json!({
            "schemaVersion": 1,
            "apps": [
                {
                    "id": "3a360d47-364b-423f-b574-9b7dcdd6a51a",
                    "name": "Antigravity IDE",
                    "launchRef": expected_launch_ref(
                        serde_json::json!({
                            "kind": "desktopEntry",
                            "value": "antigravity.desktop",
                            "executableHint": "/usr/share/antigravity/antigravity"
                        }),
                        serde_json::json!({ "kind": "windowsApp", "value": "Antigravity" })
                    ),
                    "arguments": [],
                    "processMatchers": [
                        { "kind": "name", "platform": "linux", "value": "antigravity" },
                        { "kind": "name", "platform": "windows", "value": "Antigravity.exe" }
                    ],
                    "enabled": true
                },
                {
                    "id": "613c1f4e-41b5-4f31-bda4-969f52891237",
                    "name": "Brave",
                    "launchRef": expected_launch_ref(
                        serde_json::json!({
                            "kind": "desktopEntry",
                            "value": "brave-browser.desktop",
                            "executableHint": "/usr/bin/brave-browser"
                        }),
                        serde_json::json!({ "kind": "windowsApp", "value": "Brave" })
                    ),
                    "arguments": [],
                    "processMatchers": [
                        { "kind": "name", "platform": "linux", "value": "brave" },
                        { "kind": "name", "platform": "windows", "value": "brave.exe" }
                    ],
                    "enabled": true
                }
            ]
        })
    );
}

#[test]
fn managed_state_keeps_the_persistence_save_contract_and_disk_schema() {
    let dir = tempfile::tempdir().unwrap();
    let owned = dir.path().join("owned");
    let state = WorkflowState::new(owned.clone());
    let mut workflow = WorkflowConfig::default();
    workflow.targets.truncate(1);
    workflow.targets[0].enabled = false;
    workflow.targets[0].linux = Some(LaunchRef::Executable {
        path: "/opt/antigravity/bin/antigravity".into(),
    });
    workflow.targets[0].arguments = vec!["--new-window".into()];
    workflow.targets[0].process_matchers = vec![
        ProcessMatcher::Executable {
            platform: Platform::Linux,
            path: "/opt/antigravity/bin/antigravity".into(),
        },
        ProcessMatcher::Name {
            platform: Platform::Windows,
            name: "Antigravity.exe".into(),
        },
    ];

    state
        .save_workflow(workstart_lib::config::WorkflowView::from_config(
            workflow.clone(),
            if cfg!(target_os = "windows") {
                Platform::Windows
            } else {
                Platform::Linux
            },
        ))
        .unwrap();

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(owned.join("workflow.json")).unwrap()).unwrap();
    assert_eq!(
        persisted,
        serde_json::json!({
            "schema_version": 1,
            "targets": [{
                "id": "3a360d47-364b-423f-b574-9b7dcdd6a51a",
                "name": "Antigravity IDE",
                "enabled": false,
                "linux": {
                    "kind": "executable",
                    "path": "/opt/antigravity/bin/antigravity"
                },
                "windows": {
                    "kind": "windows_app",
                    "app_name": "Antigravity"
                },
                "arguments": ["--new-window"],
                "process_matchers": [
                    {
                        "kind": "executable",
                        "platform": "linux",
                        "path": "/opt/antigravity/bin/antigravity"
                    },
                    { "kind": "name", "platform": "windows", "name": "Antigravity.exe" }
                ]
            }]
        })
    );
    assert_eq!(
        serde_json::to_value(state.get_workflow().unwrap()).unwrap(),
        serde_json::json!({
            "schemaVersion": 1,
            "apps": [{
                "id": "3a360d47-364b-423f-b574-9b7dcdd6a51a",
                "name": "Antigravity IDE",
                "launchRef": expected_launch_ref(
                    serde_json::json!({
                        "kind": "executable",
                        "value": "/opt/antigravity/bin/antigravity"
                    }),
                    serde_json::json!({ "kind": "windowsApp", "value": "Antigravity" })
                ),
                "arguments": ["--new-window"],
                "processMatchers": [
                    {
                        "kind": "executable",
                        "platform": "linux",
                        "value": "/opt/antigravity/bin/antigravity"
                    },
                    { "kind": "name", "platform": "windows", "value": "Antigravity.exe" }
                ],
                "enabled": false
            }]
        })
    );
}

#[test]
fn recovered_workflow_still_has_the_fixed_frontend_contract() {
    let dir = tempfile::tempdir().unwrap();
    let owned = dir.path().join("owned");
    std::fs::create_dir_all(&owned).unwrap();
    std::fs::write(owned.join("workflow.json"), b"not json").unwrap();
    let state = WorkflowState::new(owned);

    let response = serde_json::to_value(state.get_workflow().unwrap()).unwrap();

    assert_eq!(response["schemaVersion"], 1);
    assert_eq!(response["apps"].as_array().unwrap().len(), 2);
    assert_eq!(response.as_object().unwrap().len(), 2);
    assert!(response.get("recovery").is_none());
}
