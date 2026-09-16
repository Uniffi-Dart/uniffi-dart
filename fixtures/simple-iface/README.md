# Fixed-width handle regression

UniFFI 0.31.2 uses a `u64` handle on every target. The Dart binding must use `Uint64` at the native boundary and an opaque `int` internally, including for objects, traits, callback values and Rust futures. These handles are bit patterns, not public unsigned numeric values. No BigInt conversion is needed for them. Regenerate the component bindings and shared runtime together.

`test/handle_cases.dart` exercises object clone/free, a declared error from an object method, object arguments and returns through a callback, Rust-only traits, Rust-backed foreign traits, Dart trait identity, and a pending Rust future returning an object. The ordinary fixture test driver runs it:

```sh
cargo +1.85.1 test -p simple_iface -- --nocapture
```

For the architecture regression and failure controls, the standalone runner supports Linux x64 and optional Linux ARM32 under QEMU. Use matching host and ARM Dart SDK releases; these runs used Dart 3.13.0. It builds the native libraries and regenerates bindings itself.

```sh
python3 fixtures/simple-iface/test/run_handle_abi_cases.py --dart /path/to/host-sdk/bin/dart

rustup target add --toolchain 1.85.1 armv7-unknown-linux-gnueabihf
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/path/to/arm-linux-gnueabihf-gcc
python3 fixtures/simple-iface/test/run_handle_abi_cases.py \
  --dart /path/to/host-sdk/bin/dart \
  --arm-sdk /path/to/arm-sdk \
  --qemu /path/to/qemu-arm-static \
  --sysroot /path/to/arm-sysroot \
  --gcc-lib /path/to/arm-gcc-runtime
```

`--sysroot` must contain the ARM dynamic loader and libc under `lib`, and `--gcc-lib` the ARM GCC shared libraries under `lib`. `CARGO_TARGET_DIR` and `PUB_CACHE` select caches; `--offline` uses cached dependencies. Core dumps are disabled and each runtime case has a 90-second watchdog. A timeout fails the run.

The runner tests generated bindings unchanged, then repeats with callback and continuation handle counters seeded to `0xfedcba9800000001` to exercise nonzero upper bits and the sign bit. Finally it deliberately replaces only the failing object's method ABI with the old pointer declaration. That mutation must pass on x64 and crash at the declared error on ARM32. It is never applied to production source. Native ABI recorders independently check handle arguments, returns, clone/free-shaped calls and callbacks without dereferencing synthetic addresses.

Reports and per-case logs are written to `target/handle-results`. The standard fixture CI covers the generated runtime cases on its host; it does not run this ARM32/QEMU matrix.

## Local results

Validated on Linux x64 and emulated ARM32, Rust 1.85.1 / UniFFI 0.31.2 / Dart 3.13.0:

| Case | x64 | ARM32 |
| --- | --- | --- |
| Generated objects, typed error, callbacks, traits, pending future | Pass | Pass |
| Same cases with high callback/continuation handles | Pass | Pass |
| Reverted pointer method declaration | Pass | Expected process abort after entering the typed error case |
| ABI recorder | All 11 comparisons agree | All six Uint64 controls agree; all five pointer declarations disagree |

These are Linux JIT runs, with the ARM VM confirming four-byte pointers. The ARM32 run uses QEMU 7.2 and an ARM glibc 2.42/GCC 15.2 runtime. No physical Android/iOS or ARM32 AOT result is claimed. The failure was reproduced locally; this investigation has not identified a downstream report of that crash.
