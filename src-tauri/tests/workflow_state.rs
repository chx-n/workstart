use workstart_lib::{
    config::{LoadStatus, WorkflowState},
    domain::WorkflowConfig,
};

#[test]
fn managed_state_loads_and_saves_only_within_its_backend_directory() {
    let dir = tempfile::tempdir().unwrap();
    let state = WorkflowState::new(dir.path().join("owned"));
    assert_eq!(state.get_workflow().unwrap().status, LoadStatus::Missing);
    let mut workflow = WorkflowConfig::default();
    workflow.targets[0].enabled = false;
    state.save_workflow(&workflow).unwrap();
    assert_eq!(state.get_workflow().unwrap().workflow, workflow);
    assert!(dir.path().join("owned/workflow.json").is_file());
    let mut json = serde_json::to_value(workflow).unwrap();
    json["config_directory"] = serde_json::json!("/tmp/arbitrary");
    assert!(serde_json::from_value::<WorkflowConfig>(json).is_err());
}
