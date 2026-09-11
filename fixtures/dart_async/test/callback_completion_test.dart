import 'dart:async';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:test/test.dart';

import '../dart_async.dart';

class ByteSource implements AsyncByteSource {
  ByteSource(this.read);
  final Future<int> Function() read;
  @override
  Future<int> value() => read();
}

void main() {
  test('result lowering failure reaches the Rust future as an error', () async {
    expect(await readAsyncByte(source: ByteSource(() async => 42)), 42);
    await expectLater(
      readAsyncByte(source: ByteSource(() async => 256)),
      throwsA(isA<UnexpectedExceptionParserException>()),
    );
  }, timeout: Timeout(Duration(seconds: 10)));

  final cases = <String, (Future<int> Function(), int)>{
    'success': (() async => 42, CALL_SUCCESS),
    'invalid result': (() async => 256, CALL_UNEXPECTED_ERROR),
    'declared error': (() async => throw NotAnIntParserException(), CALL_ERROR),
    'unexpected error': (
      () async => throw StateError('test'),
      CALL_UNEXPECTED_ERROR,
    ),
    'synchronous throw': (
      () => throw StateError('test'),
      CALL_UNEXPECTED_ERROR,
    ),
  };
  for (final entry in cases.entries) {
    test('${entry.key} completes exactly once and releases its future handle', () async {
      final handle = FfiConverterCallbackInterfaceAsyncByteSource.lower(
        ByteSource(entry.value.$1),
      );
      final out = calloc<UniffiForeignFuture>();
      final done = Completer<int>();
      var completions = 0;
      final completion =
          NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
            int data,
            UniffiForeignFutureResultU8 result,
          ) {
            completions++;
            final code = result.callStatus.code;
            if (code != CALL_SUCCESS) result.callStatus.errorBuf.free();
            // Rust may synchronously release the foreign future on completion.
            out.ref.free.asFunction<UniffiForeignFutureFreeDart>()(
              out.ref.handle,
            );
            if (!done.isCompleted) done.complete(code);
          });
      try {
        asyncByteSourceValue(handle.address, completion.nativeFunction, 0, out);
        expect(await done.future.timeout(Duration(seconds: 5)), entry.value.$2);
        await Future<void>.delayed(Duration.zero);
        expect(completions, 1);
        expect(
          uniffiForeignFutureHandleMap.maybeRemove(out.ref.handle),
          isNull,
        );
      } finally {
        completion.close();
        calloc.free(out);
        asyncByteSourceFreeCallback(handle.address);
      }
    });
  }

  for (final fail in [false, true]) {
    test(
      'cancellation suppresses later ${fail ? "error" : "success"}',
      () async {
        final pending = Completer<int>();
        final handle = FfiConverterCallbackInterfaceAsyncByteSource.lower(
          ByteSource(() => pending.future),
        );
        final out = calloc<UniffiForeignFuture>();
        var completions = 0;
        final completion =
            NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
              int data,
              UniffiForeignFutureResultU8 result,
            ) {
              completions++;
              if (result.callStatus.code != CALL_SUCCESS)
                result.callStatus.errorBuf.free();
            });
        try {
          asyncByteSourceValue(
            handle.address,
            completion.nativeFunction,
            0,
            out,
          );
          final free = out.ref.free.asFunction<UniffiForeignFutureFreeDart>();
          out.ref.free.asFunction<UniffiForeignFutureFreeDart>()(
            out.ref.handle,
          );
          out.ref.free.asFunction<UniffiForeignFutureFreeDart>()(
            out.ref.handle,
          );
          if (fail) {
            pending.completeError(StateError('cancelled'));
          } else {
            pending.complete(256);
          }
          // Completing the Future schedules its continuation before this event.
          await Future<void>.delayed(Duration.zero);
          expect(completions, 0);
          expect(
            uniffiForeignFutureHandleMap.maybeRemove(out.ref.handle),
            isNull,
          );
        } finally {
          completion.close();
          calloc.free(out);
          asyncByteSourceFreeCallback(handle.address);
        }
      },
    );
  }
}
