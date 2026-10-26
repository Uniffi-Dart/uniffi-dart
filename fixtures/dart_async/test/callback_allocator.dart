import 'dart:ffi';

import 'package:ffi/ffi.dart' as ffi;

bool fail = false;
int failures = 0;

class TestAllocator implements Allocator {
  @override
  Pointer<T> allocate<T extends NativeType>(int byteCount, {int? alignment}) {
    if (fail) {
      failures++;
      throw StateError('injected native allocation failure');
    }
    return ffi.calloc.allocate<T>(byteCount, alignment: alignment);
  }

  @override
  void free(Pointer<NativeType> pointer) => ffi.calloc.free(pointer);
}

final calloc = TestAllocator();
