#![cfg(target_os = "linux")]

use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use tempfile::TempDir;
use workstart_lib::{
    domain::{AppTarget, ErrorCategory, LaunchRef, WorkflowConfig},
    platform::{
        PlatformAdapter,
        linux::{LinuxAdapter, LinuxPaths},
    },
};
use workstart_lib::{
    domain::{Platform, ProcessMatcher, RunningState},
    platform::{AppIcon, ProcessInfo, ProcessSnapshot, ProcessSpawner, ResolvedLaunch},
};

struct Fixture {
    _root: TempDir,
    adapter: LinuxAdapter,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let dirs: Vec<_> = ["user", "system"].map(|d| root.path().join(d)).into();
        for dir in &dirs {
            fs::create_dir_all(dir.join("applications")).unwrap();
        }
        let bin = root.path().join("bin");
        fs::create_dir(&bin).unwrap();
        let adapter = LinuxAdapter {
            paths: LinuxPaths {
                data_dirs: dirs,
                executable_dirs: vec![bin],
                icon_dirs: vec![root.path().join("icons")],
                locale: "sr_RS.UTF-8@latin".into(),
            },
        };
        Self {
            _root: root,
            adapter,
        }
    }
    fn exe(&self, name: &str) -> PathBuf {
        let path = self.adapter.paths.executable_dirs[0].join(name);
        fs::write(&path, b"\x7fELFfixture").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
    fn entry(&self, dir: usize, id: &str, body: &str) {
        let path = self.adapter.paths.data_dirs[dir]
            .join("applications")
            .join(id);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    fn target(&self, id: &str) -> AppTarget {
        let mut target = WorkflowConfig::default().targets.remove(0);
        target.linux = Some(LaunchRef::DesktopEntry {
            desktop_id: id.into(),
            executable_hint: None,
        });
        target
    }
}

#[test]
fn user_overrides_and_tombstones_mask_system_entries() {
    let f = Fixture::new();
    f.exe("app");
    f.entry(
        1,
        "same.desktop",
        "[Desktop Entry]\nType=Application\nName=System\nExec=app\n",
    );
    f.entry(
        0,
        "same.desktop",
        "[Desktop Entry]\nType=Application\nName=User\nExec=app\n",
    );
    f.entry(
        1,
        "gone.desktop",
        "[Desktop Entry]\nType=Application\nName=Gone\nExec=app\n",
    );
    f.entry(0, "gone.desktop", "[Desktop Entry]\nHidden=true\n");
    let apps = f.adapter.discover().unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "User");
}

#[test]
fn validates_visibility_try_exec_and_localized_names() {
    let f = Fixture::new();
    f.exe("app");
    for (id, extra) in [
        ("hidden", "Hidden=true"),
        ("nodisplay", "NoDisplay=true"),
        ("missing", "TryExec=missing"),
        ("terminal", "Terminal=true"),
        ("invalid", "Hidden=perhaps"),
    ] {
        f.entry(
            0,
            &format!("{id}.desktop"),
            &format!("[Desktop Entry]\nType=Application\nName=Bad\nExec=app\n{extra}\n"),
        );
    }
    f.entry(0, "good.desktop", "[Desktop Entry]\nType=Application\nName=Default\nName[sr]=Language\nName[sr_RS]=Country\nName[sr@latin]=Modifier\nTryExec=app\nExec=app\n[Desktop Action other]\nName=Other\n");
    let apps = f.adapter.discover().unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "Country");
}

#[test]
fn resolves_quoted_exec_and_field_codes_without_shell_interpretation() {
    let f = Fixture::new();
    let exe = f.exe("space app");
    f.entry(0, "good.desktop", &format!("[Desktop Entry]\nType=Application\nName=Some App\nIcon=app-icon\nExec=\"{}\" \"two words\" %U %i %c %k %% %d\n", exe.display()));
    let resolved = f.adapter.resolve(&f.target("good.desktop")).unwrap();
    assert_eq!(resolved.executable, exe);
    let expected: Vec<String> = vec![
        "two words".into(),
        "--icon".into(),
        "app-icon".into(),
        "Some App".into(),
        f.adapter.paths.data_dirs[0]
            .join("applications/good.desktop")
            .to_str()
            .unwrap()
            .into(),
        "%".into(),
    ];
    assert_eq!(resolved.arguments, expected);
}

#[test]
fn dbus_fallback_only_when_direct_exec_unavailable_and_arguments_empty() {
    let f = Fixture::new();
    let gtk = f.exe("gtk-launch");
    let app = f.exe("app");
    f.entry(
        0,
        "bus.desktop",
        "[Desktop Entry]\nType=Application\nName=Bus\nDBusActivatable=true\n",
    );
    let mut target = f.target("bus.desktop");
    let resolved = f.adapter.resolve(&target).unwrap();
    assert_eq!(resolved.executable, gtk);
    assert_eq!(resolved.arguments, ["bus.desktop"]);
    target.arguments.push("custom".into());
    assert_eq!(f.adapter.resolve(&target), Err(ErrorCategory::Unsupported));
    f.entry(
        0,
        "direct.desktop",
        "[Desktop Entry]\nType=Application\nName=Direct\nDBusActivatable=true\nExec=app\n",
    );
    assert_eq!(
        f.adapter
            .resolve(&f.target("direct.desktop"))
            .unwrap()
            .executable,
        app
    );
}

#[test]
fn rejects_malformed_exec_and_explicit_shells() {
    let f = Fixture::new();
    f.exe("app");
    f.exe("sh");
    for (i, exec) in [
        "app %Z",
        "app \"unfinished",
        "app %Fembedded",
        "app %U %f",
        "sh -c payload",
        "app 'shell quoting'",
    ]
    .iter()
    .enumerate()
    {
        f.entry(
            0,
            &format!("bad{i}.desktop"),
            &format!("[Desktop Entry]\nType=Application\nName=Bad\nExec={exec}\n"),
        );
    }
    assert!(f.adapter.discover().unwrap().is_empty());
    assert!(f.adapter.resolve(&f.target("bad0.desktop")).is_err());
}

#[test]
fn icons_prefer_requested_scale_and_png_then_fall_back_explicitly() {
    let f = Fixture::new();
    f.exe("app");
    f.entry(
        0,
        "app.desktop",
        "[Desktop Entry]\nType=Application\nName=App\nExec=app\nIcon=app\n",
    );
    let root = &f.adapter.paths.icon_dirs[0];
    for path in [
        "hicolor/32x32/apps/app.png",
        "hicolor/64x64/apps/app.svg",
        "hicolor/64x64/apps/app.png",
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "fixture icon").unwrap();
    }
    let mut app = f.adapter.discover().unwrap().remove(0);
    assert_eq!(
        f.adapter.icon(&app, 32, 2),
        AppIcon::File(root.join("hicolor/64x64/apps/app.png"))
    );
    app.icon = Some(
        root.join("hicolor/32x32/apps/app.png")
            .to_str()
            .unwrap()
            .into(),
    );
    assert_eq!(
        f.adapter.icon(&app, 32, 1),
        AppIcon::File(PathBuf::from(app.icon.as_ref().unwrap()))
    );
    app.icon = Some("../outside".into());
    assert_eq!(f.adapter.icon(&app, 32, 1), AppIcon::Fallback);
    app.icon = Some("missing".into());
    assert_eq!(f.adapter.icon(&app, 32, 1), AppIcon::Fallback);
}

#[test]
fn running_prefers_exact_paths_and_is_conservative_for_incomplete_or_ambiguous_snapshots() {
    let f = Fixture::new();
    let exe = f.exe("app");
    let mut target = f.target("unused.desktop");
    target.process_matchers.clear();
    target.linux = Some(LaunchRef::Executable {
        path: exe.to_str().unwrap().into(),
    });
    let process = ProcessInfo {
        executable: Some(exe.clone()),
        name: "app".into(),
    };
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![process],
                incomplete: false
            }
        ),
        RunningState::Running
    );
    assert_eq!(
        f.adapter.running(&target, &ProcessSnapshot::default()),
        RunningState::NotRunning
    );
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![],
                incomplete: true
            }
        ),
        RunningState::Unknown
    );
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: None,
                    name: "app".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Unknown
    );
    target.process_matchers.push(ProcessMatcher::Name {
        platform: Platform::Linux,
        name: "app".into(),
    });
    let other = f.exe("other");
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: Some(other),
                    name: "app".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Unknown
    );
}

#[test]
fn known_brave_aliases_and_wrapper_uncertainty() {
    let f = Fixture::new();
    let wrapper = f.exe("brave-browser-stable");
    fs::write(&wrapper, "#!/bin/bash\nfixture").unwrap();
    f.entry(
        0,
        "brave-browser.desktop",
        "[Desktop Entry]\nType=Application\nName=Brave\nExec=brave-browser-stable\n",
    );
    let target = WorkflowConfig::default().targets.remove(1);
    assert_eq!(
        f.adapter.running(&target, &ProcessSnapshot::default()),
        RunningState::Unknown
    );
    let real = f.exe("brave");
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: Some(real),
                    name: "brave".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Running
    );
}

#[test]
fn launching_keeps_custom_arguments_separate_and_inert() {
    use std::cell::RefCell;
    struct Recorder(RefCell<Vec<ResolvedLaunch>>);
    impl ProcessSpawner for Recorder {
        fn spawn(&self, launch: &ResolvedLaunch) -> Result<(), ErrorCategory> {
            self.0.borrow_mut().push(launch.clone());
            Ok(())
        }
    }
    let f = Fixture::new();
    let exe = f.exe("app");
    f.entry(
        0,
        "app.desktop",
        "[Desktop Entry]\nType=Application\nName=App\nExec=app --base\n",
    );
    let mut target = f.target("app.desktop");
    target.arguments = vec![
        "two words".into(),
        "; touch /tmp/no".into(),
        "$(payload)".into(),
        "`payload`".into(),
        "".into(),
    ];
    let recorder = Recorder(RefCell::new(vec![]));
    f.adapter.launch(&target, &recorder).unwrap();
    let calls = recorder.0.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].executable, exe);
    assert_eq!(
        calls[0].arguments,
        [vec!["--base".into()], target.arguments].concat()
    );
}

#[test]
fn resolves_default_antigravity_alternate_desktop_id_but_respects_tombstones() {
    let f = Fixture::new();
    let exe = f.exe("antigravity");
    f.entry(
        0,
        "Antigravity_IDE.desktop",
        "[Desktop Entry]\nType=Application\nName=Antigravity IDE\nExec=antigravity\n",
    );
    let target = WorkflowConfig::default().targets.remove(0);
    assert_eq!(f.adapter.resolve(&target).unwrap().executable, exe);
    f.entry(0, "antigravity.desktop", "[Desktop Entry]\nHidden=true\n");
    assert!(f.adapter.resolve(&target).is_err());
}

#[test]
fn canonical_symlinks_match_and_unreadable_process_paths_remain_unknown() {
    let f = Fixture::new();
    let exe = f.exe("app");
    let link = f._root.path().join("link");
    std::os::unix::fs::symlink(&exe, &link).unwrap();
    let mut target = f.target("unused.desktop");
    target.process_matchers.clear();
    target.linux = Some(LaunchRef::Executable {
        path: link.to_str().unwrap().into(),
    });
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: Some(exe),
                    name: "different".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Running
    );
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: Some(f._root.path().join("missing")),
                    name: "app".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Unknown
    );
}

#[test]
fn exec_two_stage_escaping_preserves_literal_metacharacters() {
    let f = Fixture::new();
    f.exe("app");
    f.entry(
        0,
        "app.desktop",
        r#"[Desktop Entry]
Type=Application
Name=App
Exec=app "a\\$b" "a\\`b" "a\\\\b" "a\\"b" "" %f
"#,
    );
    assert_eq!(
        f.adapter
            .resolve(&f.target("app.desktop"))
            .unwrap()
            .arguments,
        ["a$b", "a`b", "a\\b", "a\"b", ""]
    );
}

#[test]
fn desktop_entry_ids_flatten_subdirectories_and_user_invalid_entries_mask_system() {
    let f = Fixture::new();
    f.exe("app");
    f.entry(
        1,
        "vendor/app.desktop",
        "[Desktop Entry]\nType=Application\nName=App\nExec=app\n",
    );
    assert_eq!(f.adapter.discover().unwrap()[0].id, "vendor-app.desktop");
    f.entry(0, "vendor-app.desktop", "malformed");
    assert!(f.adapter.discover().unwrap().is_empty());
}

#[test]
fn xdg_environment_defaults_and_override_order_are_injectable() {
    let paths = LinuxPaths::from_lookup(|key| match key {
        "HOME" => Some("/home/fixture".into()),
        "XDG_DATA_HOME" => Some("/custom/user".into()),
        "XDG_DATA_DIRS" => Some("/first:relative:/second".into()),
        "PATH" => Some("/bin::relative:/usr/bin".into()),
        "LANG" => Some("fr_FR.UTF-8".into()),
        _ => None,
    });
    assert_eq!(
        paths.data_dirs,
        [
            PathBuf::from("/custom/user"),
            "/first".into(),
            "/second".into()
        ]
    );
    assert_eq!(
        paths.executable_dirs,
        [PathBuf::from("/bin"), "/usr/bin".into()]
    );
    assert_eq!(paths.locale, "fr_FR.UTF-8");
    let defaults = LinuxPaths::from_lookup(|key| (key == "HOME").then(|| "/home/fixture".into()));
    assert_eq!(
        defaults.data_dirs,
        [
            PathBuf::from("/home/fixture/.local/share"),
            "/usr/local/share".into(),
            "/usr/share".into()
        ]
    );
}

#[test]
fn proc_snapshot_reads_fixture_symlinks_and_marks_access_failures_incomplete() {
    use workstart_lib::platform::{ProcessSource, linux::ProcSource};
    let f = Fixture::new();
    let exe = f.exe("app");
    let proc_root = f._root.path().join("proc");
    fs::create_dir_all(proc_root.join("42")).unwrap();
    fs::write(proc_root.join("42/comm"), "app\n").unwrap();
    std::os::unix::fs::symlink(&exe, proc_root.join("42/exe")).unwrap();
    let source = ProcSource {
        root: proc_root.clone(),
    };
    let snapshot = source.snapshot();
    assert_eq!(
        snapshot.processes,
        [ProcessInfo {
            executable: Some(exe),
            name: "app".into()
        }]
    );
    assert!(!snapshot.incomplete);
    fs::create_dir(proc_root.join("43")).unwrap();
    fs::write(proc_root.join("43/comm"), "protected\n").unwrap();
    assert!(source.snapshot().incomplete);
    assert!(
        ProcSource {
            root: proc_root.join("missing")
        }
        .snapshot()
        .incomplete
    );
}

#[test]
fn command_builder_uses_arg_array_and_working_directory_without_starting_process() {
    use workstart_lib::platform::linux::CommandSpawner;
    let launch = ResolvedLaunch {
        executable: "/fixture/app".into(),
        arguments: vec!["a b".into(), "$(touch nope)".into(), "".into()],
        working_directory: Some("/fixture/work".into()),
        uncertain_process_identity: false,
    };
    let command = CommandSpawner::command(&launch);
    assert_eq!(command.get_program(), "/fixture/app");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["a b", "$(touch nope)", ""]
    );
    assert_eq!(
        command.get_current_dir(),
        Some(std::path::Path::new("/fixture/work"))
    );
}

#[test]
fn command_spawner_categorizes_missing_executable_without_launching_apps() {
    use workstart_lib::platform::linux::CommandSpawner;
    let f = Fixture::new();
    let launch = ResolvedLaunch {
        executable: f._root.path().join("does-not-exist"),
        arguments: vec![],
        working_directory: None,
        uncertain_process_identity: false,
    };
    assert_eq!(CommandSpawner.spawn(&launch), Err(ErrorCategory::NotFound));
}

#[test]
fn environment_wrappers_cannot_bypass_shell_rejection() {
    let f = Fixture::new();
    f.exe("env");
    f.exe("sh");
    f.entry(
        0,
        "shell.desktop",
        "[Desktop Entry]\nType=Application\nName=Bad\nExec=env sh -c payload\n",
    );
    assert!(f.adapter.discover().unwrap().is_empty());
    assert_eq!(
        f.adapter.resolve(&f.target("shell.desktop")),
        Err(ErrorCategory::Unsupported)
    );
}

#[test]
fn multiple_distinct_known_alias_paths_are_ambiguous() {
    let f = Fixture::new();
    let launcher = f.exe("brave-browser-stable");
    fs::write(launcher, "#!/bin/bash\nfixture").unwrap();
    f.entry(
        0,
        "brave-browser.desktop",
        "[Desktop Entry]\nType=Application\nName=Brave\nExec=brave-browser-stable\n",
    );
    let first = f.exe("brave");
    let second = f._root.path().join("second/brave");
    fs::create_dir(second.parent().unwrap()).unwrap();
    fs::copy(&first, &second).unwrap();
    let target = WorkflowConfig::default().targets.remove(1);
    let snapshot = ProcessSnapshot {
        processes: vec![
            ProcessInfo {
                executable: Some(first.clone()),
                name: "brave".into(),
            },
            ProcessInfo {
                executable: Some(second),
                name: "brave".into(),
            },
        ],
        incomplete: false,
    };
    assert_eq!(f.adapter.running(&target, &snapshot), RunningState::Unknown);
    let same = ProcessSnapshot {
        processes: vec![
            ProcessInfo {
                executable: Some(first.clone()),
                name: "brave".into(),
            },
            ProcessInfo {
                executable: Some(first),
                name: "brave".into(),
            },
        ],
        incomplete: false,
    };
    assert_eq!(f.adapter.running(&target, &same), RunningState::Running);
}

#[test]
fn explicit_canonical_matcher_resolves_wrapper_identity() {
    let f = Fixture::new();
    let launcher = f.exe("launcher");
    fs::write(&launcher, "#!/bin/bash\nfixture").unwrap();
    let actual = f.exe("actual");
    let mut target = f.target("unused.desktop");
    target.linux = Some(LaunchRef::Executable {
        path: launcher.to_str().unwrap().into(),
    });
    target.process_matchers = vec![ProcessMatcher::Executable {
        platform: Platform::Linux,
        path: actual.to_str().unwrap().into(),
    }];
    assert_eq!(
        f.adapter.running(
            &target,
            &ProcessSnapshot {
                processes: vec![ProcessInfo {
                    executable: Some(actual),
                    name: "actual".into()
                }],
                incomplete: false
            }
        ),
        RunningState::Running
    );
}

#[test]
#[ignore = "opt-in read-only discovery on the Fedora development host"]
fn manual_host_discovery_finds_brave_and_antigravity() {
    let adapter = LinuxAdapter {
        paths: LinuxPaths::from_environment(),
    };
    let apps = adapter.discover().unwrap();
    for name in ["brave", "antigravity"] {
        assert!(
            apps.iter()
                .any(|app| app.name.to_lowercase().contains(name)),
            "missing {name}"
        );
    }
    for target in WorkflowConfig::default().targets {
        let resolved = adapter.resolve(&target).unwrap();
        println!("{}: {}", target.name, resolved.executable.display());
    }
}
