# Callback dispatch experiments and generated bridge

The opt-in generator now emits a native companion and Dart wrappers. See [the integration guide](../../docs/callback-dispatch.md) for configuration, native-asset packaging, supported API shapes and shutdown requirements.

Run the generated bridge with `python3 fixtures/callback-thread-relay/probe/run_generated.py --dart /path/to/dart`, adding `--aot` for compiled execution. This uses generated adapters, separate endpoint/vtable state per isolate and globally distinct native callback handles. The dedicated CI workflow runs both modes.

The remainder describes the earlier handwritten relay, retained as a baseline and failure-control harness. Its limits and older device reports are separate from generated-bridge results.

## Earlier owner-dispatch experiment

The handwritten experiment extended the earlier fixture-specific synchronous relay without changing the production generator. No downstream incident has been identified in this investigation.

The original relay uses temporary queues. A nested call invoking an outer callback handle deadlocks because only the inner queue is serviced. A callback retained after its originating call returns aborts when it sends to the closed queue. Both were reproduced before changing dispatch.

This version shares one persistent queue across the owning Dart isolate. Nested synchronous calls pump that queue. Between calls, a NativeCallable.listener wakes Dart to drain it. The listener carries no callback arguments or stack pointers; native requests own their inputs and wait for results through channels. Queue and route locks are released before executing generated Dart callbacks.

Each exported call retains its own result channel, so nested pumping cannot consume another call's result. Listener drains process at most 64 jobs before yielding. Fairness under sustained load is not established by these tests.

Routes survive until native free. Graceful close refuses while handles, active calls, queued jobs or listener notifications remain. Dart closes the listener only after native close succeeds. Forced teardown and cancellation are not implemented.

## Run

From this checkout's repository root:

```sh
python3 fixtures/callback-thread-relay/probe/run.py --dart /path/to/dart
python3 fixtures/callback-thread-relay/probe/run.py --dart /path/to/dart --aot
```

Requires Python 3, Rust 1.85.1 and Dart. Use --offline for cached dependencies, --toolchain to select Rust, --timeout for the watchdog, and --cases for a comma-separated subset. CARGO_TARGET_DIR and PUB_CACHE can select isolated caches. AOT uses dart build cli to bundle native assets; dart compile exe alone omitted the native asset in our initial harness attempt.

The runner regenerates bindings using this checkout's generator and redirects native symbols to the fixture adapters. Generated Dart callbacks and converters remain unchanged except read-only registry instrumentation. Baseline, relay and state-mutation packages are isolated.

Eleven relay cases cover direct and worker/join calls, concurrent workers, clone/free, errors, nested calls, 100 synchronous repetitions, nesting through an outer handle, retained background callbacks, 50 retained clone/free cycles, and shutdown with pending work. Successful cases verify registry cleanup and orderly close. The unmodified worker callback must abort with the Dart diagnostic; a deliberately incorrect state update must fail despite returning the expected first value. Timeouts never pass these controls.

The dedicated callback probe workflow runs both modes on Linux x64 and uploads its reports, including on failure. JIT reports are in target/relay-results and AOT reports in target/relay-results-aot. See VALIDATION.md for evidence and DESIGN.md for the integration proposal.

## Limits

- Single isolate and one registration per process. Production needs per-isolate endpoints/vtables and globally distinct native handles.
- Only this fixture's synchronous callback methods and selected exports are adapted. Arbitrary callback arguments nested in records or objects are not registered automatically.
- The native companion is handwritten inside the fixture. Production requires generated ABI adapters and runtime packaging.
- Native operations move to workers. The bridge must be opt-in for APIs with known native thread requirements.
- Async Dart callbacks, foreign-future cancellation, forced shutdown and invalid-route error policies remain unresolved. Synchronously pumping native requests does not run Dart's event loop for an awaited timer/Future. Invalid protocol use can still abort.
- These changes are tested on Linux x64. The Intel macOS, iOS simulator and Android emulator reports on #186 concern the earlier prototype. No new mobile or ARM32 validation is claimed.
- Handle counters establish tested lifecycle cleanup, not general heap leak freedom.
