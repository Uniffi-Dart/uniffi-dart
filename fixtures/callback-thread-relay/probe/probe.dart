import 'dart:ffi';
import 'dart:typed_data';

import 'callback_thread_relay.dart';

// Resolve instrumentation through the same native asset as the generated API.
// Opening a second path can load another copy with independent counters/TLS.
@Native<Uint64 Function(Uint32)>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_stat',
)
external int stat(int which);
@Native<Uint64 Function()>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_background_result',
)
external int backgroundResult();
@Native<Void Function(Pointer<NativeFunction<Void Function()>>)>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_set_listener',
)
external void setListener(Pointer<NativeFunction<Void Function()>> wake);
@Native<Void Function()>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_drain',
)
external void drain();
@Native<Bool Function()>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_close',
)
external bool closeRelay();
bool requireOwner = false;
void check(bool condition, String message) {
  if (!condition) throw StateError(message);
}

class CounterSink implements Sink {
  int value;
  int calls = 0;
  CounterSink(this.value);
  @override
  int bytes(Uint8List bytes) {
    check(!requireOwner || stat(5) > 0, 'callback did not run on relay owner');
    check(bytes.length == 1 && bytes[0] == 42, 'RustBuffer payload corrupted');
    calls++;
    return ++value;
  }

  @override
  int checked(int value) {
    check(
      !requireOwner || stat(5) > 0,
      'error callback did not run on relay owner',
    );
    calls++;
    if (value == 0) throw RejectedCallbackException();
    return value + 1;
  }
}

class NestedSink extends CounterSink {
  final CounterSink inner = CounterSink(200);
  NestedSink() : super(100);
  @override
  int bytes(Uint8List bytes) {
    final outer = super.bytes(bytes);
    return outer + callBytesThread(sink: inner);
  }
}

class CrossNestedSink extends CounterSink {
  final CounterSink auxiliary = CounterSink(0);
  CrossNestedSink() : super(100);
  @override
  int bytes(Uint8List bytes) {
    final value = super.bytes(bytes);
    if (calls == 1) {
      print('ENTER nested call using outer callback handle');
      return value + callSavedThread(sink: auxiliary);
    }
    return value;
  }
}

void clean() {
  check(relayProbeHandleCount() == 0, 'generated Dart callback handles leaked');
  check(stat(0) == 0, 'native callback routes leaked');
  check(stat(1) == 0, 'native relay still active');
}

Future<void> waitForBackground(int expected) async {
  final watch = Stopwatch()..start();
  while (backgroundResult() == 0 &&
      watch.elapsed < const Duration(seconds: 5)) {
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  check(
    backgroundResult() == expected,
    'background callback did not complete correctly',
  );
}

Future<void> main(List<String> args) async {
  final mode = args.single;
  requireOwner = !mode.startsWith('baseline');
  final listener = requireOwner
      ? NativeCallable<Void Function()>.listener(drain)
      : null;
  if (listener != null) setListener(listener.nativeFunction);
  final sink = CounterSink(100);
  switch (mode) {
    case 'baseline-direct':
      check(callBytesDirect(sink: sink) == 101, 'direct result wrong');
      check(
        sink.calls == 1 && sink.value == 101,
        'direct callback state wrong',
      );
    case 'baseline-thread':
      callBytesThread(sink: sink);
      throw StateError(
        'unmodified worker-thread callback unexpectedly returned',
      );
    case 'thread':
      check(callBytesThread(sink: sink) == 101, 'thread result wrong');
      check(
        sink.calls == 1 && sink.value == 101,
        'owner state was not updated',
      );
      check(stat(2) == 1 && stat(4) == 1, 'method/free did not use relay');
    case 'direct':
      check(callBytesDirect(sink: sink) == 101, 'direct relay result wrong');
      check(sink.calls == 1, 'direct relay skipped Dart');
    case 'parallel':
      check(
        callBytesParallel(sink: sink) == 11240,
        'parallel results corrupted',
      );
      check(sink.calls == 80 && sink.value == 180, 'parallel callbacks lost');
      check(
        stat(2) == 80 && stat(4) == 1,
        'parallel dispatch/free counts wrong',
      );
    case 'clone':
      check(callCloneThread(sink: sink) == 101, 'cloned callback result wrong');
      check(sink.calls == 1, 'cloned handle did not use original Dart object');
      check(stat(3) == 1 && stat(4) == 2, 'clone/free counts wrong');
    case 'errors':
      check(
        callCheckedThread(sink: sink, value: 41) == 42,
        'checked success wrong',
      );
      bool caught = false;
      try {
        callCheckedThread(sink: sink, value: 0);
      } on CallbackException catch (error) {
        caught = error is RejectedCallbackException;
      }
      check(caught, 'declared callback error did not round trip');
      check(sink.calls == 2 && stat(4) == 2, 'error-path cleanup count wrong');
    case 'nested':
      final nested = NestedSink();
      check(
        callBytesThread(sink: nested) == 302,
        'nested callback result wrong',
      );
      check(
        nested.calls == 1 && nested.inner.calls == 1,
        'nested callbacks missing',
      );
      check(
        nested.value == 101 && nested.inner.value == 201,
        'nested state wrong',
      );
      check(stat(2) == 2 && stat(4) == 2, 'nested dispatch/free counts wrong');
    case 'repeat':
      for (var i = 0; i < 100; i++) {
        check(
          callBytesThread(sink: sink) == 101 + i,
          'repeat result wrong at $i',
        );
        clean();
      }
      check(sink.calls == 100 && sink.value == 200, 'repeat callbacks lost');
      check(stat(4) == 100, 'repeat free count wrong');
    case 'cross-nested':
      final nested = CrossNestedSink();
      check(callCrossNested(sink: nested) == 203, 'cross-nested result wrong');
      check(
        nested.calls == 2 && nested.value == 102,
        'cross-nested state wrong',
      );
    case 'retained':
      check(saveSink(sink: sink), 'save failed');
      check(stat(1) == 0 && stat(0) == 1, 'retained route missing');
      check(!closeRelay(), 'closed while Rust still owned callback');
      print('ENTER background callback after original call returned');
      fireSavedBackground();
      await waitForBackground(101);
      check(sink.calls == 1 && sink.value == 101, 'retained state wrong');
    case 'retained-clone-repeat':
      for (var i = 0; i < 50; i++) {
        check(saveSink(sink: sink), 'save failed');
        fireSavedBackgroundClone();
        await waitForBackground(101 + i);
        clean();
      }
      check(
        sink.calls == 50 && sink.value == 150,
        'retained clone state wrong',
      );
      check(
        stat(3) == 50 && stat(4) == 100,
        'retained clone/free counts wrong',
      );
    case 'shutdown-pending':
      check(saveSink(sink: sink), 'save failed');
      fireSavedBackground();
      // Keep the Dart event loop busy until the worker has queued its request.
      final watch = Stopwatch()..start();
      while (stat(6) == 0 && watch.elapsed < const Duration(seconds: 5)) {}
      check(
        stat(6) > 0 && stat(7) == 1,
        'expected queued work and outstanding notification',
      );
      check(!closeRelay(), 'closed with a pending native request');
      await waitForBackground(101);
      check(sink.calls == 1, 'pending callback lost during close attempt');
    default:
      throw ArgumentError('Unknown case $mode');
  }
  clean();
  if (listener != null) {
    final deadline = DateTime.now().add(const Duration(seconds: 5));
    while (!closeRelay()) {
      check(
        DateTime.now().isBefore(deadline),
        'relay did not drain before close',
      );
      await Future<void>.delayed(const Duration(milliseconds: 1));
    }
    check(
      stat(6) == 0 && stat(7) == 0 && stat(8) == 1,
      'unsafe shutdown state',
    );
    listener.close();
  }
  print(
    'PASS $mode: calls=${sink.calls}, handles=${relayProbeHandleCount()}, routes=${stat(0)}',
  );
}
