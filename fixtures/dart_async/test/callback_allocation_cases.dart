import 'dart:async';
import 'dart:ffi';

import 'package:test/test.dart';

import '../dart_async.dart';
import 'callback_allocator.dart' as tracker;

class Source implements AsyncByteSource {
  @override
  Future<int> value() async {
    tracker.fail = true;
    throw StateError('native allocation failure');
  }
}

void main() {
  test(
    'persistent native allocation failure still completes exactly once',
    () async {
      final handle = FfiConverterCallbackInterfaceAsyncByteSource.lower(
        Source(),
      );
      final out = tracker.calloc<UniffiForeignFuture>();
      final done = Completer<int>();
      var calls = 0;
      final completion =
          NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
            int data,
            UniffiForeignFutureResultU8 result,
          ) {
            calls++;
            expect(tracker.failures, greaterThan(0));
            expect(result.callStatus.errorBuf.len, 0);
            expect(result.callStatus.errorBuf.data, nullptr);
            tracker.fail = false;
            result.callStatus.errorBuf.free();
            done.complete(result.callStatus.code);
          });
      try {
        asyncByteSourceValue(handle, completion.nativeFunction, 0, out);
        expect(
          await done.future.timeout(Duration(seconds: 5)),
          CALL_UNEXPECTED_ERROR,
        );
        expect(calls, 1);
        expect(
          uniffiForeignFutureHandleMap.maybeRemove(out.ref.handle),
          isNull,
        );
      } finally {
        tracker.fail = false;
        completion.close();
        tracker.calloc.free(out);
        asyncByteSourceFreeCallback(handle);
      }
    },
  );
}
