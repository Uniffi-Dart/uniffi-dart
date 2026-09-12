import 'dart:ffi';

import 'package:ffi/ffi.dart' as ffi;

class TrackingAllocator implements Allocator {
  final live = <int, int>{};
  int? failAfter;

  @override
  Pointer<T> allocate<T extends NativeType>(int byteCount, {int? alignment}) {
    if (failAfter == 0) throw StateError('injected allocation failure');
    if (failAfter != null) failAfter = failAfter! - 1;
    final pointer = ffi.calloc.allocate<T>(byteCount, alignment: alignment);
    live[pointer.address] = byteCount;
    return pointer;
  }

  @override
  void free(Pointer<NativeType> pointer) {
    if (live.remove(pointer.address) == null) {
      throw StateError('free of untracked pointer');
    }
    ffi.calloc.free(pointer);
  }

  void reset() {
    failAfter = null;
    for (final address in live.keys.toList()) {
      free(Pointer<Void>.fromAddress(address));
    }
  }
}

final calloc = TrackingAllocator();
