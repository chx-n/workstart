use std::fs;
use uuid::Uuid;
use workstart_lib::{
    config::{ConfigStore, LoadStatus},
    domain::WorkflowConfig,
};

#[test]
fn missing_configuration_returns_explicit_defaults_without_creating_files() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().join("new"));
    let loaded = store.load().unwrap();
    assert_eq!(loaded.status, LoadStatus::Missing);
    assert_eq!(loaded.workflow, WorkflowConfig::default());
    assert!(!dir.path().join("new").exists());
}

#[test]
fn saves_round_trip_and_replace_existing_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().join("config"));
    let mut config = WorkflowConfig::default();
    store.save(&config).unwrap();
    config.targets[0].name = "My IDE".into();
    config.targets.reverse();
    store.save(&config).unwrap();
    let loaded = store.load().unwrap();
    assert_eq!(loaded.status, LoadStatus::Loaded);
    assert_eq!(loaded.workflow, config);
    assert_eq!(fs::read_dir(dir.path().join("config")).unwrap().count(), 1);
}

#[test]
fn corrupt_and_unsupported_files_are_preserved_and_reported() {
    for contents in [
        "{broken".to_owned(),
        "{\"schema_version\":99}".to_owned(),
        serde_json::to_string(&serde_json::json!({"schema_version":1,"targets":[]}))
            .unwrap()
            .replace("[]", "null"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workflow.json");
        fs::write(&path, &contents).unwrap();
        let store = ConfigStore::new(dir.path().to_path_buf());
        let loaded = store.load().unwrap();
        let LoadStatus::Recovered { backup_file, .. } = loaded.status else {
            panic!("missing recovery status")
        };
        assert_eq!(
            fs::read_to_string(dir.path().join(backup_file)).unwrap(),
            contents
        );
        assert!(!path.exists());
        assert_eq!(loaded.workflow, WorkflowConfig::default());
        store.save(&loaded.workflow).unwrap();
        assert_eq!(store.load().unwrap().status, LoadStatus::Loaded);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}

#[test]
fn invalid_save_keeps_last_valid_file_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    let mut config = WorkflowConfig::default();
    store.save(&config).unwrap();
    let original = fs::read(dir.path().join("workflow.json")).unwrap();
    config.targets[1].id = config.targets[0].id;
    assert!(store.save(&config).is_err());
    assert_eq!(
        fs::read(dir.path().join("workflow.json")).unwrap(),
        original
    );
}

#[cfg(unix)]
#[test]
fn failed_atomic_write_preserves_last_valid_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    let mut config = WorkflowConfig::default();
    store.save(&config).unwrap();
    let original = fs::read(dir.path().join("workflow.json")).unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();
    config.targets[0].name = "Changed".into();
    let result = store.save(&config);
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err(), "the temporary file creation must fail");
    assert_eq!(
        fs::read(dir.path().join("workflow.json")).unwrap(),
        original
    );
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn filesystem_errors_are_not_misreported_as_corruption() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("workflow.json")).unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    assert!(store.load().is_err());
    assert!(dir.path().join("workflow.json").is_dir());
}

#[test]
fn valid_escaped_configuration_is_not_rejected_by_the_file_size_guard() {
    let dir = tempfile::tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    let mut config = WorkflowConfig::default();
    let template = config.targets[0].clone();
    config.targets = (1..=64)
        .map(|id| {
            let mut target = template.clone();
            target.id = Uuid::from_u128(id);
            target.arguments = vec!["\u{1}".repeat(4096); 24];
            target
        })
        .collect();

    config.validate().unwrap();
    store.save(&config).unwrap();
    assert!(
        fs::metadata(dir.path().join("workflow.json"))
            .unwrap()
            .len()
            > 32 * 1024 * 1024
    );

    let loaded = store.load().unwrap();
    assert_eq!(loaded.status, LoadStatus::Loaded);
    assert_eq!(loaded.workflow, config);
}
