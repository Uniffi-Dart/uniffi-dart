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

/// Assert the CLI rejected its arguments during parsing: exit code 2 with
/// `needle` in stderr. Surfaces the real exit status and stderr on failure so a
/// message-wording change is debuggable rather than a bare `assert!` panic.
fn assert_usage_error(cmd: &mut Command, needle: &str) {
    let out = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "expected exit 2; stderr: {stderr}");
    assert!(stderr.contains(needle), "stderr should contain {needle:?}; stderr: {stderr}");
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
    assert_usage_error(&mut cli(), "required");
}

#[test]
fn unknown_flag_exits_two() {
    assert_usage_error(cli().arg("--definitely-not-a-flag"), "unknown option");
}

#[test]
fn option_missing_value_exits_two() {
    // `--out-dir` with no following value.
    assert_usage_error(cli().arg("--out-dir"), "requires a value");
}

#[test]
fn config_missing_value_exits_two() {
    assert_usage_error(cli().arg("--config"), "requires a value");
}

#[test]
fn duplicate_out_dir_exits_two() {
    // The `--out-dir=x` form also exercises the inline `=value` consumption path
    // (the first sets the slot, so reaching the duplicate error proves `=x` was
    // consumed and stored, not dropped).
    assert_usage_error(cli().args(["--out-dir=x", "--out-dir=y"]), "more than once");
}

#[test]
fn duplicate_crate_exits_two() {
    // `--crate` has its own hand-written duplicate guard (it holds a `String`,
    // not the shared `set_once` slot), so it needs its own case.
    assert_usage_error(cli().args(["--crate", "a", "--crate", "b"]), "more than once");
}

#[test]
fn library_flag_is_accepted_not_unknown() {
    // `--library` exists only for `uniffi-bindgen` parity; downstreams pass it.
    // `--library=x` hits its `reject_inline` arm ("takes no value"); if the flag
    // regressed it would fall through to the unknown-option arm instead.
    assert_usage_error(cli().arg("--library=x"), "takes no value");
}

#[test]
fn multiple_sources_exits_two() {
    assert_usage_error(cli().args(["a.so", "b.so"]), "more than one source path");
}

#[test]
fn boolean_flag_rejects_inline_value() {
    // `--no-format=…` must be rejected, not silently ignored.
    assert_usage_error(cli().arg("--no-format=yes"), "takes no value");
}
