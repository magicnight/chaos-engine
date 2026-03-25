use std::process::Command;

fn cargo_run(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run chaos")
}

#[test]
fn test_status_shows_all_sources() {
    let output = cargo_run(&["status"]);
    // Status output may go to stdout or stderr depending on formatting
    let all_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());
    assert!(all_output.contains("GDELT"), "Missing GDELT in output");
    assert!(all_output.contains("USGS"), "Missing USGS in output");
    assert!(all_output.contains("NOAA"), "Missing NOAA in output");
    assert!(all_output.contains("WHO"), "Missing WHO in output");
    assert!(all_output.contains("YFinance"), "Missing YFinance in output");
}

#[test]
fn test_unknown_source_exits_with_error() {
    let output = cargo_run(&["source", "nonexistent"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown source")
            || stderr.contains("unknown source")
            || stderr.contains("not found"),
        "Should indicate unknown source"
    );
}

/// Hit real API — run with: cargo test --test sweep_test -- --ignored
#[test]
#[ignore]
fn test_sweep_json_structure() {
    let output = cargo_run(&["sweep", "--json"]);
    assert!(output.status.success(), "sweep should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value =
        serde_json::from_str(&stdout).expect("sweep output must be valid JSON");

    assert!(data["chaos"]["version"].is_string());
    assert!(data["chaos"]["timestamp"].is_string());
    assert_eq!(data["chaos"]["sourcesQueried"].as_u64().unwrap(), 5);
    assert!(data["chaos"]["sourcesOk"].as_u64().unwrap() >= 1);
    assert!(data["sources"].is_object());
    assert!(data["timing"].is_object());
}

/// Hit real API
#[test]
#[ignore]
fn test_single_source_usgs() {
    let output = cargo_run(&["source", "usgs"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value =
        serde_json::from_str(&stdout).expect("source output must be valid JSON");
    assert_eq!(data["source"].as_str().unwrap(), "USGS");
    assert_eq!(data["status"].as_str().unwrap(), "ok");
}

#[test]
#[ignore]
fn test_parallel_isolation() {
    let output = cargo_run(&["sweep", "--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&stdout).expect("Not valid JSON");

    let queried = data["chaos"]["sourcesQueried"].as_u64().unwrap();
    assert_eq!(queried, 5);

    let timing = data["timing"].as_object().unwrap();
    assert_eq!(timing.len(), 5, "All 5 sources must have timing data");
}
