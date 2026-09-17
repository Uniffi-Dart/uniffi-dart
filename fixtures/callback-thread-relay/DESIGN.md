# Recommended direction for #161

Use an opt-in native dispatch runtime with one queue per Dart isolate. Synchronous pumping and listener notifications service the same queue.

## Demonstrated improvements

- Synchronous generated callbacks preserve return types and the user's original mutable Dart objects. The Rust operation runs on a worker while the entering Dart thread pumps requests.
- Sharing the queue across nested calls fixes the reproduced outer-handle deadlock.
- Retained native handles route to the same owner after the creating call returns. Listener notifications let the ordinary Dart event loop service them.
- Methods, clone and free use the same dispatch protocol. Graceful shutdown waits for handles, jobs and posted notifications before closing the listener.

These are fixture-level results, not completed generator support.

## Integration order

1. Retain the expanded fixture and failure controls. Request platform runs for this extension; the earlier #186 reports validate only the earlier code.
2. Extract the runtime and generate ABI adapters for callback vtables and exported calls. Install native proxy vtables once, retain each endpoint's original Dart vtables, and route all callback arguments and clone/free operations through the endpoint. Adapters must follow the actual UniFFI wire layout, including fixed-width handles. The separate ARM32 finding still applies.
3. Define per-isolate registration and cleanup. Native handles must distinguish both endpoint and Dart-local handle: separate isolates can allocate the same local number. The prototype's process-global vtable and thread-local depth cannot provide this isolation.
4. Add an explicit asynchronous invocation path for operations waiting on async Dart callbacks. Return control to Dart while native work runs, preserve immediate foreign-future setup, and test cancellation before dispatch, during the Future, and after completion. Do not silently change all existing synchronous APIs to return Futures.
5. Define refusal and teardown behavior. Busy graceful close is rejected here, but forced shutdown, invalid handles and abandoned workers need a production lifetime/error policy. Closing a listener early is not cancellation.

The native adapter could be packaged alongside the existing Rust library using exported C ABI functions. Generated C adapters versus a Rust companion remain an implementation choice to validate. This fixture does not prove either packaging workflow or require editing downstream application code now.

## Alternatives

- `NativeCallable.isolateLocal` retains the owner-thread restriction and cannot fix the worker-thread abort.
- `NativeCallable.listener` alone cannot preserve synchronous return/status/clone semantics. Here it only wakes the owner; native request storage and completion channels retain those semantics.
- `NativeCallable.isolateGroupBound` is experimental and restricts callbacks to trivially shareable state. It cannot directly preserve the generator's arbitrary user objects in per-isolate registries.
- A thread-affine native operation blocking the Dart owner thread while its worker waits for that isolate creates a scheduling cycle. Such APIs need an asynchronous or library-specific entry strategy, or an explicit restriction. The worker relay must be opt-in.

Primary references:
- https://api.dart.dev/dart-ffi/NativeCallable/NativeCallable.isolateLocal.html
- https://api.dart.dev/dart-ffi/NativeCallable/NativeCallable.listener.html
- https://api.dart.dev/dart-ffi/NativeCallable/NativeCallable.isolateGroupBound.html

This integration sequence is our design conclusion, not a guarantee from Dart documentation.
