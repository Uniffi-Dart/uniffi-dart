# CI coverage

The Testing workflow runs workspace tests and Clippy on Rust 1.85, stable and
nightly. Failures on all three toolchains fail their jobs. The matrix continues
running other toolchains so each result remains available. Clippy allows
`large_const_arrays` because UniFFI 0.31's generated scaffolding constructs its
static metadata from const `MetadataBuffer` values; other warnings remain errors.

The Nix flake check validates repository formatting. Dev-shell jobs verify that
Rust, Cargo, cargo-nextest and Dart can run, stopping at the first failed tool.
Runtime coverage comes from the Testing workflow.

The downstream workflow generates bindings using this checkout and runs Payjoin
and BDK tests. Routine BDK runs report their passing, failed and skipped counts
in the job summary. They do not require live Bitcoin testnet servers.

To run BDK integration tests, manually run **Test Downstream**, select the branch
to check, and supply Bitcoin testnet Electrum and Esplora API URLs. These inputs
must point to testnet, which the downstream fixtures use. That run enables all
BDK tests and fails if any tests are skipped, if no tests pass, or if the test
report is incomplete. It requires reachable servers for the duration of the run.
