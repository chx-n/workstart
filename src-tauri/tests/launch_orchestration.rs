use std::sync::{Arc, Mutex};

use workstart_lib::{
    domain::{AppTarget, ErrorCategory, RunningState, WorkflowConfig},
    orchestration::{
        LaunchLog, RotatingLog, TargetLaunchResult, TargetRuntime, launch_from_stored_ids,
    },
};

#[derive(Default)]
struct Runtime {
    running: Mutex<Vec<(uuid::Uuid, RunningState)>>,
    launched: Mutex<Vec<uuid::Uuid>>,
    failures: Mutex<Vec<(uuid::Uuid, ErrorCategory)>>,
}

impl Runtime {
    fn with_running(self, target: &AppTarget, state: RunningState) -> Self {
        self.running.lock().unwrap().push((target.id, state));
        self
    }

    fn with_failure(self, target: &AppTarget, error: ErrorCategory) -> Self {
        self.failures.lock().unwrap().push((target.id, error));
        self
    }
}

impl TargetRuntime for Runtime {
    fn running(&self, target: &AppTarget) -> RunningState {
        self.running
            .lock()
            .unwrap()
            .iter()
            .find(|(id, _)| *id == target.id)
            .map(|(_, state)| *state)
            .unwrap_or(RunningState::NotRunning)
    }

    fn launch(&self, target: &AppTarget) -> Result<(), ErrorCategory> {
        self.launched.lock().unwrap().push(target.id);
        self.failures
            .lock()
            .unwrap()
            .iter()
            .find(|(id, _)| *id == target.id)
            .map(|(_, error)| *error)
            .map_or(Ok(()), Err)
    }
}

#[derive(Default)]
struct Logs(Mutex<Vec<String>>);

impl LaunchLog for Logs {
    fn record(&self, entry: &workstart_lib::orchestration::LogEntry) -> std::io::Result<()> {
        self.0
            .lock()
            .unwrap()
            .push(serde_json::to_string(entry).unwrap());
        Ok(())
    }
}

#[test]
fn stored_ids_launch_independently_while_running_targets_are_skipped() {
    let workflow = WorkflowConfig::default();
    let first = &workflow.targets[0];
    let second = &workflow.targets[1];
    let runtime = Arc::new(
        Runtime::default()
            .with_running(first, RunningState::Running)
            .with_running(second, RunningState::Unknown),
    );
    let logs = Logs::default();

    let results = launch_from_stored_ids(
        &workflow,
        &[first.id.to_string(), second.id.to_string()],
        runtime.as_ref(),
        &logs,
    );

    assert_eq!(
        serde_json::to_value(&results).unwrap(),
        serde_json::json!([
            { "targetId": first.id, "outcome": "skippedAlreadyRunning" },
            { "targetId": second.id, "outcome": "started" }
        ])
    );
    assert_eq!(*runtime.launched.lock().unwrap(), vec![second.id]);
}

#[test]
fn unknown_or_invalid_ids_and_one_target_failure_do_not_block_other_targets() {
    let workflow = WorkflowConfig::default();
    let first = &workflow.targets[0];
    let second = &workflow.targets[1];
    let runtime = Runtime::default().with_failure(second, ErrorCategory::NotFound);
    let logs = Logs::default();
    let missing = uuid::Uuid::max().to_string();

    let results = launch_from_stored_ids(
        &workflow,
        &[first.id.to_string(), missing.clone(), second.id.to_string()],
        &runtime,
        &logs,
    );

    assert_eq!(results[0].outcome, "started");
    assert_eq!(results[1].target_id, missing);
    assert_eq!(results[1].outcome, "failed");
    assert_eq!(results[1].error_category, Some(ErrorCategory::NotFound));
    assert_eq!(results[2].outcome, "failed");
    assert_eq!(results[2].error_category, Some(ErrorCategory::NotFound));
    let launched = runtime.launched.lock().unwrap();
    assert_eq!(launched.len(), 2);
    assert!(launched.contains(&first.id));
    assert!(launched.contains(&second.id));
}

#[test]
fn logs_are_structured_and_never_include_argument_values() {
    let mut workflow = WorkflowConfig::default();
    workflow.targets[0].arguments = vec!["--token=very-secret-value".into()];
    let target = &workflow.targets[0];
    let logs = Logs::default();

    launch_from_stored_ids(
        &workflow,
        &[target.id.to_string()],
        &Runtime::default(),
        &logs,
    );

    let line = logs.0.lock().unwrap().pop().unwrap();
    assert!(line.contains("launch_result"));
    assert!(line.contains(&target.id.to_string()));
    assert!(!line.contains("very-secret-value"));
    assert!(!line.contains("--token"));
}

#[test]
fn rotating_log_keeps_a_bounded_number_of_bounded_local_files() {
    let directory = tempfile::tempdir().unwrap();
    let log = RotatingLog::with_limits(directory.path().into(), 300, 3).unwrap();
    let result = TargetLaunchResult::started(uuid::Uuid::max().to_string());

    for _ in 0..5 {
        log.record_result(&result).unwrap();
    }

    let files = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 3);
    assert!(
        files
            .iter()
            .all(|entry| entry.metadata().unwrap().len() <= 300)
    );
    assert!(
        files
            .iter()
            .all(|entry| entry.file_name() != "launch.log.3")
    );
}
