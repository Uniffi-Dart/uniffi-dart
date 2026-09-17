# Opt-in synchronous callback dispatch

Generated Dart callbacks normally use `Pointer.fromFunction`, which aborts when Rust invokes them from another thread. The dispatcher keeps callback execution in the owning Dart isolate and moves native entry calls to workers. Dart-facing calls and callback methods remain synchronous; the original callback objects and their mutable state remain in that isolate.

This is experimental and disabled by default. No downstream report of the original worker-thread crash has been identified in this investigation.

## Enable and package

Add this to the configuration used to generate your bindings:

```toml
[bindings.dart]
callback_dispatch = true
```

Generation emits Dart wrappers and a standalone Rust crate under `uniffi_dispatch/`. The crate uses only Rust's standard library and supports Rust 1.85. Build it for the same architecture and platform as the original UniFFI library:

```sh
cargo +1.85.1 build --release --manifest-path path/to/generated/uniffi_dispatch/Cargo.toml
# For cross compilation, also select the target and configure its linker.
```

The library basename is `<namespace>_dart_dispatch`, with the platform's normal library prefix and extension. Bundle it as a second Native Asset alongside the original Rust library. Its asset name is the original configured asset name plus `_dispatch`; for example:

```dart
output.assets.code.add(CodeAsset(
  package: input.packageName,
  name: 'uniffi:example_dispatch',
  linkMode: DynamicLoadingBundled(),
  file: pathToBuiltCompanion,
));
```

The generated crate README records the exact library basename and full asset ID. The existing original-library asset stays registered too. Generated wrappers resolve original entry points through `Native.addressOf`; the companion does not statically link a second copy of the original library.

The generated probe includes a complete two-asset hook and tests the result as both JIT and an AOT application built with `dart build cli`.

## Supported scope

This version supports one component per generated package on 64-bit native targets, with synchronous Rust functions and synchronous Dart callback interfaces/traits. It handles worker/join calls, nested callbacks, retained callbacks between native calls, clone/free, declared callback errors and multiple Dart isolates using the same native library.

Generation rejects async Rust APIs, async Dart callbacks, Rust object types, callback values returned to Dart, callback values nested in records/enums or passed through callback methods, and external component types. It also rejects direct callback enum/duration/timestamp/custom wire types until their existing converter mismatches are resolved. The companion refuses to compile for 32-bit targets. These limits are checked before producing a bridge that would silently use an unsupported ABI or lifetime path.

Do not mix dispatched and ordinary bindings for the same Rust component in one process: UniFFI installs callback vtables process-wide. Each isolate using dispatched bindings gets its own copied Dart vtables and local handle registry. Native handles include routing through a process-wide registry, so two isolates can both allocate local handle 1 safely.

Opting in moves native entry calls, including calls without callback arguments, to fresh worker threads. Libraries that require calls on a particular thread must not use this mode. Thread creation adds overhead. Reentrant callbacks still need the Rust application to release any locks that a nested call would acquire.

## Lifetime and shutdown

While a Dart call waits for native work, it pumps its own queue. Between calls, `NativeCallable.listener` wakes the owner isolate to drain requests. Requests keep their inputs alive until completion, and no queue or route lock is held while invoking Dart. Dispatch uses active endpoint identity rather than assuming an isolate always runs on the same OS thread.

If lowering a later argument fails before a native call starts, generated wrappers roll back callback registrations from that call. Once the native call starts, Rust owns those handles and releases them through its callback free path.

After Rust has dropped every callback it retained, close the dispatcher in that isolate:

```dart
while (!closeCallbackDispatcher()) {
  await Future<void>.delayed(const Duration(milliseconds: 1));
}
```

In an application, use a timeout or an explicit native shutdown acknowledgement rather than retrying indefinitely. `false` means a callback handle, native call, queued request or listener notification remains. A successful close removes the native endpoint, releases its copied vtables and closes the Dart listener. Close is idempotent. Do not call the bindings after successful close.

The listener deliberately keeps its isolate alive until close. Forced isolate termination, abandoned workers and cancellation are not supported; killing an isolate while Rust retains its callbacks can still invalidate native callback pointers. Supporting async callbacks requires a separate invocation path that yields to Dart's event loop, not a synchronous queue pump.

## Reproduce

From the repository root:

```sh
python3 fixtures/callback-thread-relay/probe/run_generated.py --dart /path/to/dart
python3 fixtures/callback-thread-relay/probe/run_generated.py --dart /path/to/dart --aot
```

The runner builds the original library, generates the bridge, builds the companion and packages both native assets. It exercises 12 scenarios, including two concurrent isolates, 100 repeated calls and 50 retained clone/free cycles. Removing callback-registration rollback makes the failed-lowering shutdown test fail. A state-mutation control must fail even though it returns the expected scalar value. Timeouts fail every case.

The older handwritten relay runner remains as a baseline with the unmodified worker-thread abort control. The generated runner does not use that relay or replace generated native symbols with fixture-specific adapters.

Local validation uses Linux x64, Dart 3.13.0, Rust 1.85.1 and UniFFI 0.31.2. Fresh device runs are needed; results previously reported for the handwritten prototype do not validate this generated bridge.

Primary Dart API references: [Native.addressOf](https://api.dart.dev/dart-ffi/Native/addressOf.html) and [NativeCallable.listener](https://api.dart.dev/dart-ffi/NativeCallable/NativeCallable.listener.html).
