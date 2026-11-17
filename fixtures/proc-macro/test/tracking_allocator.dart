import 'dart:ffi';

import 'package:ffi/ffi.dart' as ffi;

/// Tracks the native allocations of an instrumented copy of the generated
/// bindings (see borrowed_bytes_allocation_test.dart), whose
/// `import "package:ffi/ffi.dart"` is rewritten to hide `calloc` and `using`.
class TrackingAllocator implements Allocator {
  final live = <int, int>{};
  final allocatedSizes = <int>[];

  @override
  Pointer<T> allocate<T extends NativeType>(int byteCount, {int? alignment}) {
    final pointer = ffi.calloc.allocate<T>(byteCount, alignment: alignment);
    live[pointer.address] = byteCount;
    allocatedSizes.add(byteCount);
    return pointer;
  }

  @override
  void free(Pointer<NativeType> pointer) {
    if (live.remove(pointer.address) == null) {
      throw StateError('free of untracked pointer');
    }
    ffi.calloc.free(pointer);
  }
}

final calloc = TrackingAllocator();

/// `package:ffi`'s `using` defaults its allocator to `calloc` as resolved in
/// `package:ffi` itself, so hiding `calloc` alone would not reach the per-call
/// arena the generated code opens. Re-export `using` bound to the tracker.
R using<R>(R Function(ffi.Arena) computation, [Allocator? wrappedAllocator]) =>
    ffi.using(computation, wrappedAllocator ?? calloc);
