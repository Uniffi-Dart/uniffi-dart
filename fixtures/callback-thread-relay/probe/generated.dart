import 'dart:ffi';
import 'dart:isolate';
import 'dart:typed_data';

import 'callback_thread_relay.dart';

@Native<Uint64 Function()>(
  assetId: 'package:relay_probe/uniffi:callback_thread_relay',
  symbol: 'relay_background_result',
)
external int backgroundResult();
void check(bool value, String message) {
  if (!value) throw StateError(message);
}

class BrokenPayload extends ArgumentPayload {
  @override
  RustBuffer lower() => throw StateError('expected later argument failure');
  @override
  int allocationSize() => throw StateError('expected later argument failure');
  @override
  int write(Uint8List buf) =>
      throw StateError('expected later argument failure');
}

class Counter implements Sink {
  int value;
  int calls = 0;
  Counter(this.value);
  @override
  int bytes(Uint8List bytes) {
    check(bytes.length == 1 && bytes[0] == 42, 'callback payload corrupted');
    calls++;
    return ++value;
  }

  @override
  int checked(int value) {
    calls++;
    if (value == 0) throw RejectedCallbackException();
    return value + 1;
  }
}

class Nested extends Counter {
  final inner = Counter(200);
  Nested() : super(100);
  @override
  int bytes(Uint8List bytes) =>
      super.bytes(bytes) + callBytesThread(sink: inner);
}

class CrossNested extends Counter {
  final auxiliary = Counter(0);
  CrossNested() : super(100);
  @override
  int bytes(Uint8List bytes) {
    final result = super.bytes(bytes);
    return calls == 1 ? result + callSavedThread(sink: auxiliary) : result;
  }
}

Future<void> close() async {
  final deadline = DateTime.now().add(const Duration(seconds: 5));
  while (!closeCallbackDispatcher()) {
    check(DateTime.now().isBefore(deadline), 'dispatcher failed to drain');
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  check(closeCallbackDispatcher(), 'close is not idempotent');
}

Future<void> retained(Counter sink, {bool clone = false}) async {
  check(saveSink(sink: sink), 'save failed');
  check(!closeCallbackDispatcher(), 'closed with a retained callback');
  final expected = sink.value + 1;
  if (clone) {
    fireSavedBackgroundClone();
  } else {
    fireSavedBackground();
  }
  final deadline = DateTime.now().add(const Duration(seconds: 5));
  while (backgroundResult() == 0) {
    check(DateTime.now().isBefore(deadline), 'retained callback stalled');
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  check(
    backgroundResult() == expected && sink.value == expected,
    'retained state/result mismatch',
  );
}

Future<void> isolateWorker((SendPort, int) args) async {
  final (port, initial) = args;
  try {
    final sink = Counter(initial);
    for (var i = 1; i <= 30; i++) {
      check(
        callBytesThread(sink: sink) == initial + i,
        'cross-isolate handle collision',
      );
      await Future<void>.delayed(const Duration(milliseconds: 1));
    }
    check(sink.calls == 30, 'isolate callback state lost');
    await close();
    port.send(initial + 30);
  } catch (e, st) {
    port.send('$e\n$st');
  }
}

Future<void> main(List<String> args) async {
  final mode = args.single;
  final sink = Counter(100);
  switch (mode) {
    case 'direct':
      check(
        callBytesDirect(sink: sink) == 101 && sink.value == 101,
        'direct callback',
      );
    case 'thread':
      check(
        callBytesThread(sink: sink) == 101 && sink.value == 101,
        'thread callback',
      );
    case 'parallel':
      check(
        callBytesParallel(sink: sink) == 11240 &&
            sink.value == 180 &&
            sink.calls == 80,
        'parallel callbacks',
      );
    case 'clone':
      check(
        callCloneThread(sink: sink) == 101 && sink.calls == 1,
        'clone callback',
      );
    case 'errors':
      check(callCheckedThread(sink: sink, value: 41) == 42, 'success result');
      var caught = false;
      try {
        callCheckedThread(sink: sink, value: 0);
      } on RejectedCallbackException {
        caught = true;
      }
      check(caught && sink.calls == 2, 'typed callback error');
    case 'nested':
      final nested = Nested();
      check(
        callBytesThread(sink: nested) == 302 && nested.inner.calls == 1,
        'nested callback',
      );
    case 'cross-nested':
      final nested = CrossNested();
      check(
        callCrossNested(sink: nested) == 203 && nested.calls == 2,
        'outer handle callback',
      );
    case 'retained':
      await retained(sink);
    case 'retained-clone':
      for (var i = 0; i < 50; i++) {
        await retained(sink, clone: true);
      }
      check(sink.calls == 50, 'retained clone callbacks lost');
    case 'repeat':
      for (var i = 1; i <= 100; i++) {
        check(callBytesThread(sink: sink) == 100 + i, 'repeated callback');
      }
    case 'rollback':
      var caught = false;
      try {
        callWithPayload(sink: sink, payload: BrokenPayload());
      } on StateError catch (e) {
        caught = e.message == 'expected later argument failure';
      }
      check(caught && sink.calls == 0, 'lowering failure reached Rust');
      check(
        closeCallbackDispatcher(),
        'failed lowering leaked its native route',
      );
    case 'isolates':
      final port = ReceivePort();
      await Isolate.spawn(isolateWorker, (port.sendPort, 1000));
      await Isolate.spawn(isolateWorker, (port.sendPort, 2000));
      final values = await port
          .take(2)
          .toList()
          .timeout(const Duration(seconds: 15));
      port.close();
      check(
        values.contains(1030) && values.contains(2030),
        'isolate routing: $values',
      );
    default:
      throw ArgumentError(mode);
  }
  await close();
  print('PASS generated $mode');
}
