# Generated callback dispatch validation

The draft is based directly on main e2dd2bb7180609184d0c70ddde0353f5e11d4c3b. The opt-in generator emits the Dart wrappers and a standalone native companion; the generated runner does not use the handwritten fixture adapters.

On 2026-09-17, Linux x64, Dart 3.13.0, Rust 1.85.1 and UniFFI 0.31.2 passed 12 positive scenarios in both JIT and compiled AOT: direct, worker/join, parallel, clone/free, declared errors, nested calls, nested calls through an outer handle, retained callbacks, retained clone/free cycles, repeated calls, concurrent isolates and rollback after argument lowering fails. Both negative controls fail for the intended assertion: a state mutation that still returns the expected scalar, and removal of callback-registration rollback. Timeouts never pass. This makes 14 runtime cases per execution mode.

The generated native companion passes three shutdown tests and Clippy with warnings denied. The tests cover an active call, a pending listener notification and another isolate's live routes. The generator's scope checks reject unsupported API shapes before generation. All three generator unit tests, 45 existing Dart callback/async tests with dispatch disabled, and generator/fixture MSRV Clippy checks pass.

Recorded reports: [generated JIT](results/generated-linux-x64-jit.json), [generated AOT](results/generated-linux-x64-aot.json). Their commit field identifies the previous published head; the changes field records the working tree containing the generated implementation tested here. Build and package setup steps are recorded separately from runtime cases.

See [the integration guide](../../docs/callback-dispatch.md) for scope, packaging and shutdown. These results do not validate async callbacks, forced isolate termination, Rust object lifetimes, 32-bit targets or mobile devices. Earlier device reports concern the handwritten prototype below.

## Earlier handwritten owner-dispatch validation

The following results predate the generated integration. That experiment did not change production generator code.

Environment: Linux x64, Dart 3.13.0, Rust 1.85.1, UniFFI 0.31.2. Both JIT and compiled AOT runs pass all 11 positive relay scenarios, the unmodified same-thread baseline, the expected wrong-thread abort, and the state-mutation rejection control: 14 runtime cases per execution mode. Build, generator and package setup checks are recorded separately in the reports.

| Added case | Original per-call queue | Persistent owner queue |
| --- | --- | --- |
| Nested native call reuses outer callback handle | Reaches test marker, then 12-second watchdog timeout | Returns 203 and updates the original object twice |
| Native callback after creating call returned | SendError panic in extern C; process abort (-6) | Returns 101 through the generated Dart callback; original state updated |
| 50 retained clone/free cycles | Not separately run | 50 clone callbacks, 100 free callbacks; no handles/routes left after each cycle |
| Close with pending request and listener notification | No close protocol | Close refuses; callback completes after yielding; final close succeeds |

The timeout was observed before changing the relay and was not accepted as a passing runtime case. The original retained-callback abort was checked for both the entry marker and SendError. The baseline source and logs are preserved in the root checkout under review/follow-ups/callback-owner-dispatch.

Every successful relay case checks an empty generated callback registry, zero native routes and no active relay. Final close additionally checks no queued jobs, no outstanding listener notification and closed native state before closing the Dart listener. This proves tested handle lifecycle behavior, not general heap leak freedom or forced-teardown safety.

All existing direct, worker/join, parallel, clone/free, declared-error, nested and repeated-call cases continue to pass in both modes. The new queue supports retained synchronous callbacks; it does not establish support for async Dart callback methods or cancellation.

Rust 1.85.1 Clippy for the fixture and all targets passes with warnings denied. Rust/Dart formatting and whitespace checks pass. AOT uses dart build cli so the native asset is bundled; the initial dart compile exe attempt omitted the native asset and failed the same-thread baseline before callback testing. That harness problem was corrected rather than accepted as an expected abort.

Recorded reports: [JIT](results/linux-x64-jit.json), [AOT](results/linux-x64-aot.json). Fresh runs on 2026-09-16 tested clean commit 816ea89 from the local experiment/callback-owner-dispatch branch. This draft carries the same fixture implementation. The subsequent changes update documentation, copy those reports and add the dedicated CI workflow.

New behavior is validated locally on Linux x64. The Intel macOS/iOS-simulator/Android-emulator results reported on #186 concern the earlier prototype. No new physical-device, ARM32, multi-isolate, async-cancellation or thread-affine-native validation is claimed.
