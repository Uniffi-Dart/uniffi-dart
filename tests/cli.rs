//! Integration coverage for the `uniffi_bindgen_dart` CLI — the argv entry point
//! (`uniffi_dart::main`) that downstream `uniffi-bindgen.rs` helpers forward to.
//!
//! These exercise the argument parser end-to-end through the built binary; they
//! need neither a compiled cdylib nor a Dart toolchain, since every case here is
//! resolved during parsing, before any generation runs.
#![cfg(feature = "binary")]

use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_uniffi_bindgen_dart"))
}

#[test]
fn version_prints_and_exits_zero() {
    let out = cli().arg("--version").output().unwrap();
    assert!(out.status.success(), "--version should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("uniffi_bindgen_dart"), "version line missing; stdout: {stdout}");
}

#[test]
fn help_prints_usage_and_exits_zero() {
    for flag in ["--help", "-h"] {
        let out = cli().arg(flag).output().unwrap();
        assert!(out.status.success(), "`{flag}` should exit 0");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("USAGE:"),
            "`{flag}` should print usage to stdout"
        );
    }
}

#[test]
fn missing_required_args_exits_two() {
    // Neither a source path nor `--out-dir`.
    let out = cli().output().unwrap();
    assert_eq!(out.status.code(), Some(2), "missing args should exit 2");
    assert!(String::from_utf8_lossy(&out.stderr).contains("required"));
}

#[test]
fn unknown_flag_exits_two() {
    let out = cli().arg("--definitely-not-a-flag").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown option"));
}

#[test]
fn option_missing_value_exits_two() {
    // `--out-dir` with no following value.
    let out = cli().arg("--out-dir").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("requires a value"));
}

#[test]
fn duplicate_option_exits_two() {
    let out = cli().args(["--out-dir", "a", "--out-dir", "b", "lib.so"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("more than once"));
}

#[test]
fn boolean_flag_rejects_inline_value() {
    // `--no-format=…` must be rejected, not silently ignored.
    let out = cli().arg("--no-format=yes").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("takes no value"));
}
