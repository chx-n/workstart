use serde_json::Value;

#[test]
fn package_metadata_matches_the_initial_release() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    assert_eq!(env!("CARGO_PKG_LICENSE"), "MIT");
}

#[test]
fn tauri_configuration_defines_secure_fixed_windows() {
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri.conf.json should contain valid JSON");

    assert_eq!(config["identifier"], "io.github.chxnn.workstart");
    assert_eq!(config["version"], "0.1.0");

    let windows = config["app"]["windows"]
        .as_array()
        .expect("app.windows should be an array");
    assert_eq!(windows.len(), 2);

    let main = window(windows, "main");
    assert_eq!(main["width"], 440);
    assert_eq!(main["height"], 320);
    assert_eq!(main["resizable"], false);
    assert_eq!(main["visible"], true);

    let settings = window(windows, "settings");
    assert_eq!(settings["width"], 760);
    assert_eq!(settings["height"], 600);
    assert_eq!(settings["resizable"], false);
    assert_eq!(settings["visible"], false);

    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("a CSP should be configured");
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("object-src 'none'"));
    assert!(csp.contains("base-uri 'none'"));
    assert!(csp.contains("frame-ancestors 'none'"));
    assert!(csp.contains("form-action 'none'"));
    assert!(!csp.contains("unsafe-eval"));
    assert!(!csp.contains("unsafe-inline"));
    assert!(!csp.contains("https:"));
}

#[test]
fn capability_is_limited_to_the_application_windows_without_shell_access() {
    let capability = include_str!("../capabilities/default.json");
    let capability_json: Value =
        serde_json::from_str(capability).expect("default capability should contain valid JSON");

    assert_eq!(
        capability_json["windows"],
        serde_json::json!(["main", "settings"])
    );
    assert_eq!(capability_json["permissions"], serde_json::json!([]));
    assert!(!capability.contains("shell"));
}

fn window<'a>(windows: &'a [Value], label: &str) -> &'a Value {
    windows
        .iter()
        .find(|window| window["label"] == label)
        .unwrap_or_else(|| panic!("missing {label} window"))
}
