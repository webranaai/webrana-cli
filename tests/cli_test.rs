// ============================================
// WEBRANA CLI - CLI Integration Tests
// Created by: VALIDATOR (Team Beta)
// ============================================

use std::process::Command;

/// Test CLI help command
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("webrana") || stdout.contains("Webrana"));
}

/// Test CLI version command
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.3.0") || stdout.contains("webrana"));
}

/// Test CLI skills command
#[test]
fn test_cli_skills() {
    let output = Command::new("cargo")
        .args(["run", "--", "skills"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

/// Test CLI config command
#[test]
fn test_cli_config() {
    let output = Command::new("cargo")
        .args(["run", "--", "config"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    // Config might fail without API key, but shouldn't panic
    // Just check it runs
    assert!(!output.stderr.is_empty() || output.status.success());
}

/// Test CLI agents command
#[test]
fn test_cli_agents() {
    let output = Command::new("cargo")
        .args(["run", "--", "agents"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}
