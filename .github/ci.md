# CI coverage

The Testing workflow runs workspace tests and Clippy on Rust 1.85, stable and
nightly. Failures on all three toolchains fail their jobs. The matrix continues
running other toolchains so each result remains available. Clippy allows
`large_const_arrays` because UniFFI 0.31's generated scaffolding constructs its
static metadata from const `MetadataBuffer` values; other warnings remain errors.

The Nix flake check validates repository formatting. Dev-shell jobs verify that
Rust, Cargo, cargo-nextest and Dart can run, stopping at the first failed tool.
Runtime coverage comes from the Testing workflow.

The existing downstream workflow generates bindings using this checkout and runs
Payjoin and BDK tests. Those suites can skip network integration tests, so a
successful downstream job does not establish that live-server integration ran.
Downstream coverage policy is separate from failure propagation in this PR.
