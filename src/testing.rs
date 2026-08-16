use std::fs::{copy, create_dir_all, File};
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::tempdir;
use uniffi_testing::UniFFITestHelper;

use crate::gen;

// A source to compile for a test
#[derive(Debug)]
pub struct CompileSource {
    pub udl_path: Utf8PathBuf,
    pub config_path: Option<Utf8PathBuf>,
}

/// Test execution options
#[derive(Debug, Clone, Default)]
pub struct TestConfig {
    /// Custom output directory for test files
    pub custom_output_dir: Option<Utf8PathBuf>,
    /// Preserve test files after completion
    pub no_delete: bool,
    /// Delay in seconds after test failure (0 = no delay; None = default)
    pub failure_delay_secs: Option<u64>,
}

impl TestConfig {
    /// Build from environment variables
    /// - UNIFFI_DART_TEST_DIR: custom output dir
    /// - UNIFFI_DART_NO_DELETE: preserve files
    /// - UNIFFI_DART_FAILURE_DELAY: failure delay (seconds)
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(test_dir) = std::env::var("UNIFFI_DART_TEST_DIR") {
            config.custom_output_dir = Some(Utf8PathBuf::from(test_dir));
        }
        if std::env::var("UNIFFI_DART_NO_DELETE").is_ok() {
            config.no_delete = true;
        }
        if let Ok(delay_str) = std::env::var("UNIFFI_DART_FAILURE_DELAY") {
            if let Ok(delay) = delay_str.parse::<u64>() {
                config.failure_delay_secs = Some(delay);
            }
        }

        config
    }

    pub fn with_no_delete(mut self, no_delete: bool) -> Self {
        self.no_delete = no_delete;
        self
    }

    pub fn with_output_dir<P: Into<Utf8PathBuf>>(mut self, dir: P) -> Self {
        self.custom_output_dir = Some(dir.into());
        self
    }

    pub fn with_failure_delay(mut self, delay_secs: u64) -> Self {
        self.failure_delay_secs = Some(delay_secs);
        self
    }
}

/// Run a test with default options (env vars honored)
///
/// Env overrides:
/// - UNIFFI_DART_TEST_DIR
/// - UNIFFI_DART_NO_DELETE
/// - UNIFFI_DART_FAILURE_DELAY
pub fn run_test(fixture: &str, udl_path: &str, config_path: Option<&str>) -> Result<()> {
    run_test_with_config(fixture, udl_path, config_path, &TestConfig::from_env())
}

/// Run a test with explicit configuration
pub fn run_test_with_config(
    fixture: &str,
    udl_path: &str,
    config_path: Option<&str>,
    test_config: &TestConfig,
) -> Result<()> {
    run_test_impl(fixture, udl_path, config_path, test_config)
}

/// Run a test with an explicit output directory (convenience wrapper)
pub fn run_test_with_output_dir(
    fixture: &str,
    udl_path: &str,
    config_path: Option<&str>,
    custom_output_dir: Option<&Utf8Path>,
) -> Result<()> {
    let config = TestConfig {
        custom_output_dir: custom_output_dir.map(|p| p.to_owned()),
        ..Default::default()
    };
    run_test_impl(fixture, udl_path, config_path, &config)
}

/// Test execution (core implementation)
fn run_test_impl(
    fixture: &str,
    udl_path: &str,
    config_path: Option<&str>,
    test_config: &TestConfig,
) -> Result<()> {
    // Resolve project root (cargo may change CWD when running tests)
    let project_root = find_project_root()?;

    let script_path = Utf8Path::new(".").canonicalize_utf8()?;
    let test_helper = UniFFITestHelper::new(fixture)?;

    // Function-scope guard to keep temp dir alive until function end
    let mut _temp_guard: Option<_> = None;

    // Resolve output dir: custom → temp (with optional preservation)
    let out_dir = if let Some(custom_dir) = &test_config.custom_output_dir {
        let resolved_dir = if custom_dir.is_absolute() {
            custom_dir.clone()
        } else {
            project_root.join(custom_dir)
        };
        create_dir_all(&resolved_dir)?;
        test_helper.create_out_dir(&resolved_dir, &script_path)?
    } else {
        let temp_dir = tempdir()?;
        let out_dir = test_helper.create_out_dir(temp_dir.path(), &script_path)?;
        if test_config.no_delete {
            // Keep temp directory alive when no_delete is set
            std::mem::forget(temp_dir);
        } else {
            _temp_guard = Some(temp_dir);
        }
        out_dir
    };

    let udl_path = Utf8Path::new(".").canonicalize_utf8()?.join(udl_path);
    let config_path = if let Some(path) = config_path {
        Some(Utf8Path::new(".").canonicalize_utf8()?.join(path))
    } else {
        None
    };

    println!("{out_dir}");

    if test_config.no_delete {
        println!("Test files will be preserved after completion (no-delete mode)");
    }

    // Get package name from config (default to "uniffi")
    let package_name = if let Some(config_path) = config_path.as_deref() {
        // Try to read package name from config
        if let Ok(config_content) = std::fs::read_to_string(config_path) {
            if let Ok(config_toml) = toml::from_str::<toml::Value>(&config_content) {
                config_toml
                    .get("bindings")
                    .and_then(|b| b.get("dart"))
                    .and_then(|d| d.get("package_name"))
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "uniffi".to_string())
            } else {
                "uniffi".to_string()
            }
        } else {
            "uniffi".to_string()
        }
    } else {
        "uniffi".to_string()
    };

    let mut pubspec = File::create(out_dir.join("pubspec.yaml"))?;
    pubspec.write_all(
        format!(
            r#"
name: {}
description: testing module for uniffi
version: 1.0.0

environment:
  sdk: '>=3.10.0'
dev_dependencies:
  test: ^1.24.3
dependencies:
  ffi: ^2.0.1
  code_assets: any
  hooks: any
"#,
            package_name
        )
        .as_bytes(),
    )?;
    pubspec.flush()?;
    let test_outdir = out_dir.join("test");
    create_dir_all(&test_outdir)?;

    // Create hook directory for Native Assets
    let hook_dir = out_dir.join("hook");
    create_dir_all(&hook_dir)?;

    // Copy cdylib and get its path
    test_helper.copy_cdylib_to_out_dir(&out_dir)?;
    let cdylib_path = test_helper.cdylib_path()?;
    let cdylib_filename = cdylib_path.file_name().unwrap();

    // Get the asset ID (should match what's in generated Dart code)
    // Format: "uniffi:{cdylib_name}" (without package prefix - that's added by Dart automatically)
    let cdylib_stem = cdylib_path.file_stem().unwrap().trim_start_matches("lib"); // Remove "lib" prefix on Unix

    // Create Native Assets build hook using modern hooks package
    let mut build_hook = File::create(hook_dir.join("build.dart"))?;
    build_hook.write_all(
        format!(
            r#"import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

// Generated by uniffi-dart – this is the full asset ID that Dart will use
// Dart automatically prefixes the 'name' with 'package:$packageName/', so we construct the full ID here
String _makeAssetId(String packageName) => 'package:$packageName/uniffi:{}';

void main(List<String> args) async {{
  await build(args, (input, output) async {{
    // The native library is in the package root (copied by test infrastructure)
    final libPath = input.packageRoot.resolve('{}');

    // Register the asset - Dart will automatically prefix the name with package:$packageName/
    // So if name is "uniffi:hello_world", the full ID becomes "package:uniffi/uniffi:hello_world"
    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'uniffi:{}',  // Dart prefixes this with "package:$packageName/"
        linkMode: DynamicLoadingBundled(),
        file: libPath,
      ),
    );

    // Verify the asset ID matches what's in generated code
    print('Registered asset: ${{_makeAssetId(input.packageName)}}');
  }});
}}
"#,
            cdylib_stem, cdylib_filename, cdylib_stem
        )
        .as_bytes(),
    )?;
    build_hook.flush()?;
    gen::generate_dart_bindings(
        &udl_path,
        config_path.as_deref(),
        Some(&out_dir),
        &test_helper.cdylib_path()?,
        false, // library_mode
        None,  // crate_name filter (library mode only)
        true,  // try_format
    )?;

    // Copy fixture test files to output directory
    let test_glob_pattern = "test/*.dart";
    for file in glob::glob(test_glob_pattern)?.filter_map(Result::ok) {
        let filename = file.file_name().expect("bad filename").to_str().expect("non-UTF8 filename");
        copy(&file, test_outdir.join(filename))?;
    }

    // Best effort formatting
    let mut format_command = Command::new("dart");
    format_command.current_dir(&out_dir).arg("format").arg(".");
    match format_command.spawn().and_then(|mut c| c.wait()) {
        Ok(status) if status.success() => {}
        Ok(_) | Err(_) => {
            println!("WARNING: dart format unavailable or failed; continuing with tests anyway");
            if std::env::var("CI").is_err() {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    // Run tests
    let mut command = Command::new("dart");
    command.current_dir(&out_dir).arg("test");
    let status = command.spawn()?.wait()?;
    if !status.success() {
        println!("FAILED");

        // Optional delay after failure (skipped on CI)
        let delay_secs = test_config.failure_delay_secs.unwrap_or(2);
        if delay_secs > 0 && std::env::var("CI").is_err() {
            println!("Waiting {} seconds before cleanup...", delay_secs);
            thread::sleep(Duration::from_secs(delay_secs));
        }

        bail!("running `dart` to run test script failed ({:?})", command);
    }
    Ok(())
}

/// Locate the workspace root:
/// - CARGO_WORKSPACE_ROOT if set
/// - ascend until a Cargo.toml with [workspace]
fn find_project_root() -> Result<Utf8PathBuf> {
    if let Ok(ws_root) = std::env::var("CARGO_WORKSPACE_ROOT") {
        if let Some(p) = Utf8Path::from_path(std::path::Path::new(&ws_root)) {
            return Ok(p.to_owned());
        }
    }

    let mut current = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Utf8Path::from_path(&current)
                        .ok_or_else(|| anyhow::anyhow!("Project root path is not valid UTF-8"))
                        .map(|p| p.to_owned());
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent.to_owned();
        } else {
            break;
        }
    }

    Utf8Path::from_path(&std::env::current_dir()?)
        .ok_or_else(|| anyhow::anyhow!("Current directory path is not valid UTF-8"))
        .map(|p| p.to_owned())
}

pub fn get_compile_sources() -> Result<Vec<CompileSource>> {
    todo!("Not implemented")
}
