use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_PNG_ICONS: &[(&str, u32)] = &[
    ("32x32.png", 32),
    ("128x128.png", 128),
    ("128x128@2x.png", 256),
    ("512x512.png", 512),
];

#[test]
fn bundle_metadata_and_platform_targets_match_the_release_contract() {
    let base = config("tauri.conf.json");
    let linux = config("tauri.linux.conf.json");
    let windows = config("tauri.windows.conf.json");

    assert_eq!(base["productName"], "Workstart");
    assert_eq!(base["version"], "0.1.0");
    assert_eq!(base["identifier"], "io.github.chxnn.workstart");
    assert_eq!(base["bundle"]["publisher"], "chxnn");
    assert_eq!(base["bundle"]["license"], "MIT");
    assert_eq!(base["bundle"]["licenseFile"], "../LICENSE");
    assert_eq!(base["bundle"]["category"], "Productivity");
    assert_eq!(base["bundle"]["createUpdaterArtifacts"], false);
    assert_eq!(
        base["bundle"]["icon"],
        serde_json::json!([
            "icons/32x32.png",
            "icons/128x128.png",
            "icons/128x128@2x.png",
            "icons/512x512.png",
            "icons/icon.ico"
        ])
    );

    assert_eq!(linux["bundle"]["targets"], serde_json::json!(["rpm"]));
    assert_eq!(linux["bundle"]["linux"]["rpm"]["epoch"], 0);
    assert_eq!(linux["bundle"]["linux"]["rpm"]["release"], "1");
    assert_eq!(
        linux["bundle"]["linux"]["rpm"]["desktopTemplate"],
        "packaging/linux/workstart.desktop.hbs"
    );

    let desktop_template = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("packaging/linux/workstart.desktop.hbs"),
    )
    .expect("read RPM desktop template");
    assert!(desktop_template.contains("Categories=Office;Utility;"));
    assert!(desktop_template.contains("Icon={{icon}}"));
    assert!(desktop_template.contains("Terminal=false"));

    assert_eq!(windows["bundle"]["targets"], serde_json::json!(["nsis"]));
    assert_eq!(
        windows["bundle"]["windows"]["webviewInstallMode"],
        serde_json::json!({ "type": "downloadBootstrapper", "silent": true })
    );
    assert_eq!(
        windows["bundle"]["windows"]["nsis"]["installMode"],
        "currentUser"
    );
    assert_eq!(
        windows["bundle"]["windows"]["nsis"]["startMenuFolder"],
        "Workstart"
    );
    assert_eq!(
        windows["bundle"]["windows"]["nsis"]["installerIcon"],
        "icons/icon.ico"
    );
}

#[test]
fn release_config_does_not_enable_signing_or_forbidden_integrations() {
    let base = config("tauri.conf.json");
    let windows = config("tauri.windows.conf.json");
    let package = config_at(workspace_root().join("package.json"));
    let cargo_manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");
    let gitignore = fs::read_to_string(workspace_root().join(".gitignore"))
        .expect("read repository .gitignore");

    assert!(base["app"].get("trayIcon").is_none());
    assert!(base.get("plugins").is_none());
    assert!(
        windows["bundle"]["windows"]
            .get("certificateThumbprint")
            .is_none()
    );
    assert!(windows["bundle"]["windows"].get("signCommand").is_none());
    assert!(
        gitignore
            .lines()
            .any(|line| line == "src-tauri/tauri.signing.conf.json"),
        "the documented local signing override must remain ignored"
    );

    let dependency_text = ["dependencies", "devDependencies"]
        .into_iter()
        .filter_map(|key| package[key].as_object())
        .flat_map(|dependencies| dependencies.keys())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in ["autostart", "global-shortcut", "updater"] {
        assert!(
            !dependency_text.contains(forbidden),
            "forbidden integration dependency found: {forbidden}"
        );
        assert!(
            !cargo_manifest.contains(forbidden),
            "forbidden Rust integration found: {forbidden}"
        );
    }
}

#[test]
fn generated_icons_have_expected_dimensions_and_alpha_channels() {
    for (name, expected_size) in EXPECTED_PNG_ICONS {
        let data = fs::read(icon_path(name)).unwrap_or_else(|error| panic!("read {name}: {error}"));
        let (width, height, color_type) = png_metadata(&data);
        assert_eq!((width, height), (*expected_size, *expected_size), "{name}");
        assert_eq!(color_type, 6, "{name} must be RGBA");
    }

    let source = fs::read(icon_path("icon.png")).expect("read source icon");
    let (width, height, color_type) = png_metadata(&source);
    assert_eq!((width, height), (1254, 1254));
    assert_eq!(color_type, 6, "source icon must remain RGBA");
}

#[test]
fn windows_icon_contains_the_required_multiresolution_frames() {
    let data = fs::read(icon_path("icon.ico")).expect("read Windows icon");
    assert!(data.len() >= 6, "ICO header is truncated");
    assert_eq!(&data[0..4], &[0, 0, 1, 0], "ICO header");

    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    assert!(data.len() >= 6 + count * 16, "ICO directory is truncated");
    let sizes = (0..count)
        .map(|index| {
            let width = data[6 + index * 16];
            if width == 0 { 256 } else { u16::from(width) }
        })
        .collect::<Vec<_>>();

    for required in [16, 24, 32, 48, 64, 128, 256] {
        assert!(
            sizes.contains(&required),
            "ICO is missing {required}x{required}"
        );
    }
}

#[test]
fn package_scripts_are_explicitly_unsigned_and_architecture_scoped() {
    let package = config_at(workspace_root().join("package.json"));
    assert_eq!(
        package["scripts"]["package:fedora:unsigned"],
        "tauri build --target x86_64-unknown-linux-gnu --bundles rpm --no-sign"
    );
    assert_eq!(
        package["scripts"]["package:windows:unsigned"],
        "tauri build --target x86_64-pc-windows-msvc --bundles nsis --no-sign"
    );
    assert_eq!(
        package["scripts"]["build:debug"],
        "tauri build --debug --no-bundle"
    );
}

fn config(name: &str) -> Value {
    config_at(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
}

fn config_at(path: PathBuf) -> Value {
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri should be inside the workspace")
        .to_path_buf()
}

fn icon_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("icons")
        .join(name)
}

fn png_metadata(data: &[u8]) -> (u32, u32, u8) {
    assert!(data.len() >= 26, "PNG header is truncated");
    assert_eq!(&data[0..8], b"\x89PNG\r\n\x1a\n", "PNG signature");
    assert_eq!(&data[12..16], b"IHDR", "IHDR must be the first chunk");
    (
        u32::from_be_bytes(data[16..20].try_into().expect("width bytes")),
        u32::from_be_bytes(data[20..24].try_into().expect("height bytes")),
        data[25],
    )
}
