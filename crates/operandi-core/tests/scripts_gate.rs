//! The test gate scripts report failure when Cargo fails (Astra P1-001).
//!
//! `scripts/test-all.sh` used to pipe Cargo's build output straight into
//! the reader that lists test programs, so a program that failed to
//! compile or link was left out and the script still passed. These tests
//! run the real script with a fake Cargo (`CARGO_BIN`) and check its exit
//! code.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// A fake Cargo whose `--no-run` build lists one test program (a script
/// that exits `test_status`) and then exits `build_status`.
fn run_test_all(build_status: i32, test_status: i32) -> (i32, String) {
    let dir = std::env::temp_dir().join(format!(
        "operandi-scripts-gate-{}-{build_status}-{test_status}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("fake-test");
    write_executable(
        &exe,
        &format!("#!/bin/sh\necho 'test result: ok. 1 passed; 0 failed'\nexit {test_status}\n"),
    );
    let artifact = format!(
        r#"{{"reason":"compiler-artifact","executable":"{}","profile":{{"test":true}},"target":{{"name":"fake","kind":["test"]}},"manifest_path":"{}"}}"#,
        exe.display(),
        dir.join("Cargo.toml").display()
    );
    let message =
        r#"{"reason":"compiler-message","message":{"rendered":"error: linking failed\n"}}"#;
    let cargo = dir.join("fake-cargo");
    write_executable(
        &cargo,
        &format!("#!/bin/sh\nprintf '%s\\n' '{artifact}' '{message}'\nexit {build_status}\n"),
    );
    let out = Command::new("sh")
        .arg(repo_root().join("scripts/test-all.sh"))
        .env("CARGO_BIN", &cargo)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), text)
}

#[test]
fn test_all_fails_with_cargos_exit_code_when_the_build_fails() {
    // The build listed a program but still failed (one target didn't link).
    let (code, text) = run_test_all(17, 0);
    assert_eq!(code, 17, "{text}");
    assert!(text.contains("cargo exit 17"), "{text}");
    assert!(
        text.contains("linking failed"),
        "compiler messages shown: {text}"
    );
    assert!(!text.contains("tests passed"), "no tests run: {text}");
}

#[test]
fn test_all_fails_when_a_test_program_fails() {
    let (code, text) = run_test_all(0, 1);
    assert_ne!(code, 0, "{text}");
    assert!(text.contains("=== FAILED: fake-test"), "{text}");
}

#[test]
fn test_all_passes_when_everything_passes() {
    let (code, text) = run_test_all(0, 0);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("1 tests passed"), "{text}");
}

/// check-fast.sh's scope for a set of changed paths, from the Cargo lines
/// a fake Cargo was asked to run.
fn check_fast_scope(changed: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "operandi-check-fast-{}-{}",
        std::process::id(),
        changed.len()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let cargo = dir.join("fake-cargo");
    write_executable(&cargo, "#!/bin/sh\necho \"cargo $*\"\n");
    let out = Command::new("sh")
        .arg(repo_root().join("scripts/check-fast.sh"))
        .env("CARGO_BIN", &cargo)
        .env("CHECK_FAST_CHANGED", changed)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Astra P10-001: a change to a root fixture tests the whole workspace,
/// since core and MCP tests read them too.
#[test]
fn check_fast_tests_the_workspace_for_fixture_changes() {
    let app = check_fast_scope("crates/operandi-app/src/main.rs");
    assert!(app.contains("clippy --locked -p operandi-app"), "{app}");
    let fixture = check_fast_scope("fixtures/node-materials/noise.json");
    assert!(fixture.contains("--workspace"), "{fixture}");
    let mixed = check_fast_scope("fixtures/a.obj\ncrates/operandi-app/src/main.rs");
    assert!(mixed.contains("--workspace"), "{mixed}");
    let docs = check_fast_scope("docs/PROGRESS.md");
    assert!(docs.contains("fmt was the only check"), "{docs}");
}
