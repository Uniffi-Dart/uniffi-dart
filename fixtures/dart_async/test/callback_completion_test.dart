import 'dart:async';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:test/test.dart';

import '../dart_async.dart';

class ByteSource implements AsyncByteSource {
  ByteSource(this.read);
  final Future<int> Function() read;
  @override
  Future<int> value() => read();
}

class ThrowingToString implements Exception {
  @override
  String toString() => throw StateError('toString failed');
}

class InputSource implements AsyncInputSource {
  @override
  Future<int> value(String input) async => throw StateError('must not run');
}

void main() {
  test('failed argument lift completes with an error', () async {
    final handle = FfiConverterCallbackInterfaceAsyncInputSource.lower(
      InputSource(),
    );
    final out = calloc<UniffiForeignFuture>();
    final invalidUtf8 = toRustBuffer(Uint8List.fromList([0xff]));
    final done = Completer<int>();
    var calls = 0;
    final completion =
        NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
          int data,
          UniffiForeignFutureResultU8 result,
        ) {
          calls++;
          result.callStatus.errorBuf.free();
          done.complete(result.callStatus.code);
        });
    try {
      asyncInputSourceValue(
        handle.address,
        invalidUtf8,
        completion.nativeFunction,
        0,
        out,
      );
      expect(
        await done.future.timeout(Duration(seconds: 5)),
        CALL_UNEXPECTED_ERROR,
      );
      expect(calls, 1);
      expect(uniffiForeignFutureHandleMap.maybeRemove(out.ref.handle), isNull);
    } finally {
      completion.close();
      calloc.free(out);
      asyncInputSourceFreeCallback(handle.address);
    }
  });
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
    'declared error with invalid field': (
      () async => throw InvalidByteParserException(256),
      CALL_UNEXPECTED_ERROR,
    ),
    'unexpected error': (
      () async => throw StateError('test'),
      CALL_UNEXPECTED_ERROR,
    ),
    'synchronous throw': (
      () => throw StateError('test'),
      CALL_UNEXPECTED_ERROR,
    ),
    'error with throwing toString': (
      () async => throw ThrowingToString(),
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
      var handleReleased = false;
      final completion =
          NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
            int data,
            UniffiForeignFutureResultU8 result,
          ) {
            completions++;
            final code = result.callStatus.code;
            if (code != CALL_SUCCESS) result.callStatus.errorBuf.free();
            // The trampoline must release the handle before completing.
            try {
              uniffiForeignFutureHandleMap.get(out.ref.handle);
            } catch (_) {
              handleReleased = true;
            }
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
        expect(handleReleased, isTrue);
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

  test('stale handle completes with an error instead of escaping', () async {
    final handle = FfiConverterCallbackInterfaceAsyncByteSource.lower(
      ByteSource(() async => 42),
    );
    asyncByteSourceFreeCallback(handle.address);
    final out = calloc<UniffiForeignFuture>();
    final done = Completer<int>();
    final completion =
        NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
          int data,
          UniffiForeignFutureResultU8 result,
        ) {
          final code = result.callStatus.code;
          if (code != CALL_SUCCESS) result.callStatus.errorBuf.free();
          if (!done.isCompleted) done.complete(code);
        });
    try {
      asyncByteSourceValue(handle.address, completion.nativeFunction, 0, out);
      expect(
        await done.future.timeout(Duration(seconds: 5)),
        CALL_UNEXPECTED_ERROR,
      );
    } finally {
      completion.close();
      calloc.free(out);
    }
  });

  for (final fail in [false, true]) {
    test(
      'cancelled future still completes ${fail ? "error" : "success"} exactly once',
      () async {
        final pending = Completer<int>();
        final handle = FfiConverterCallbackInterfaceAsyncByteSource.lower(
          ByteSource(() => pending.future),
        );
        final out = calloc<UniffiForeignFuture>();
        var completions = 0;
        int? completedCode;
        final completion =
            NativeCallable<UniffiForeignFutureCompleteU8>.isolateLocal((
              int data,
              UniffiForeignFutureResultU8 result,
            ) {
              completions++;
              completedCode = result.callStatus.code;
              if (result.callStatus.code != CALL_SUCCESS) {
                result.callStatus.errorBuf.free();
              }
            });
        try {
          asyncByteSourceValue(
            handle.address,
            completion.nativeFunction,
            0,
            out,
          );
          final free = out.ref.free.asFunction<UniffiForeignFutureFreeDart>();
          free(out.ref.handle);
          free(out.ref.handle);
          if (fail) {
            pending.completeError(StateError('cancelled'));
          } else {
            pending.complete(42);
          }
          // Completing the Future schedules its continuation before this event.
          await Future<void>.delayed(Duration.zero);
          // Rust keeps the completion slot until Dart completes, even after
          // cancellation, so completing here releases it.
          expect(completions, 1);
          expect(completedCode, fail ? CALL_UNEXPECTED_ERROR : CALL_SUCCESS);
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
