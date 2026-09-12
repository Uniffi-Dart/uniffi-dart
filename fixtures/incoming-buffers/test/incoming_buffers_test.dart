import 'dart:ffi';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:test/test.dart';

import '../incoming_buffers.dart';

class Consumer implements BufferConsumer {
  List<Uint8List> saved = [];
  @override
  int consume(Uint8List first, Uint8List second) {
    saved = [first, second];
    return first.length + second.length;
  }
}

class AsyncConsumer implements AsyncBufferConsumer {
  List<Uint8List> saved = [];
  @override
  Future<int> consume(Uint8List first, Uint8List second) async {
    await Future<void>.delayed(Duration.zero);
    saved = [first, second];
    return first.length + second.length;
  }
}

class ThrowingHandler extends UniffiRustCallStatusErrorHandler {
  @override
  Exception lift(RustBuffer buffer) => throw FormatException('bad error');
}

void expectPacket(Packet packet) {
  expect(packet.bytes, List.filled(64, 42));
  expect(packet.groups, [List.filled(8, 11), List.filled(16, 22)]);
  expect(packet.optional, List.filled(4, 33));
}

void main() {
  ensureInitialized();

  test('synchronous results release their Rust buffers', () {
    final before = liveBytes();
    for (var i = 0; i < 100; i++) {
      expect(makeBytes(), List.filled(64, 42));
      expect(makeString(), 'x' * 64);
      expectPacket(makePacket());
    }
    expect(liveBytes(), before);
  });

  test('byte readers copy borrowed storage, including nested values', () {
    final packet = Packet(
      bytes: Uint8List.fromList([1, 2]),
      groups: [
        Uint8List.fromList([3]),
      ],
      optional: Uint8List.fromList([4]),
    );
    final storage = Uint8List(FfiConverterPacket.allocationSize(packet));
    FfiConverterPacket.write(packet, storage);
    final copied = FfiConverterPacket.read(storage).value;
    storage.fillRange(0, storage.length, 0);
    expect(copied.bytes, [1, 2]);
    expect(copied.groups, [
      [3],
    ]);
    expect(copied.optional, [4]);
  });

  test('async results release buffers and preserve nested bytes', () async {
    await makePacketAsync();
    final before = liveBytes();
    for (var i = 0; i < 20; i++) {
      expectPacket(await makePacketAsync());
    }
    expect(liveBytes(), before);
  });

  test('declared errors release buffers and preserve byte payloads', () {
    final before = liveBytes();
    for (var i = 0; i < 20; i++) {
      expect(
        failResult,
        throwsA(
          isA<RejectedOwnershipException>().having(
            (e) => e.payload,
            'payload',
            List.filled(32, 7),
          ),
        ),
      );
    }
    expect(liveBytes(), before);
  });

  test('async errors release their buffers', () async {
    await expectLater(failAsync(), throwsA(isA<RejectedOwnershipException>()));
    final before = liveBytes();
    for (var i = 0; i < 20; i++) {
      await expectLater(
        failAsync(),
        throwsA(isA<RejectedOwnershipException>()),
      );
    }
    expect(liveBytes(), before);
  });

  test('panic messages release their buffers', () {
    expect(panicResult, throwsA(isA<UniffiInternalError>()));
    final before = liveBytes();
    for (var i = 0; i < 3; i++) {
      expect(panicResult, throwsA(isA<UniffiInternalError>()));
    }
    expect(liveBytes(), before);
  });

  test('a throwing result lifter still releases its input', () {
    final before = liveBytes();
    final buffer = RustBuffer.alloc(64);
    expect(
      () => rustCallWithLifter<int, RustBuffer>(
        (_) => buffer,
        (_) => throw FormatException('bad result'),
      ),
      throwsFormatException,
    );
    expect(liveBytes(), before);
  });

  for (final handler in [NullRustCallStatusErrorHandler(), ThrowingHandler()]) {
    test('error status owns its buffer when using ${handler.runtimeType}', () {
      final before = liveBytes();
      final status = calloc<RustCallStatus>();
      try {
        status.ref.code = CALL_ERROR;
        status.ref.errorBuf = RustBuffer.alloc(64);
        expect(
          () => checkCallStatus(handler, status),
          throwsA(isA<Exception>()),
        );
        expect(liveBytes(), before);
      } finally {
        calloc.free(status);
      }
    });
  }

  test('an empty panic message still releases allocated storage', () {
    final before = liveBytes();
    final status = calloc<RustCallStatus>();
    try {
      final buffer = RustBuffer.alloc(64);
      buffer.len = 0;
      status.ref.code = CALL_UNEXPECTED_ERROR;
      status.ref.errorBuf = buffer;
      expect(
        () => checkCallStatus(NullRustCallStatusErrorHandler(), status),
        throwsA(
          isA<UniffiInternalError>().having(
            (e) => e.panicMessage,
            'message',
            'Rust panic',
          ),
        ),
      );
      expect(liveBytes(), before);
    } finally {
      calloc.free(status);
    }
  });

  test('error status does not lift an invalid result', () {
    final before = liveBytes();
    var lifted = false;
    expect(
      () => rustCallWithLifter<int, RustBuffer?>(
        (status) {
          status.ref.code = CALL_ERROR;
          status.ref.errorBuf = RustBuffer.alloc(64);
          return null;
        },
        (_) {
          lifted = true;
          return 0;
        },
      ),
      throwsA(isA<UniffiInternalError>()),
    );
    expect(lifted, isFalse);
    expect(liveBytes(), before);
  });

  test('callback inputs are released and saved byte values remain usable', () {
    final consumer = Consumer();
    sendBuffers(consumer: consumer);
    final before = liveBytes();
    for (var i = 0; i < 20; i++) {
      expect(sendBuffers(consumer: consumer), 96);
    }
    expect(liveBytes(), before);
    expect(consumer.saved, [List.filled(64, 4), List.filled(32, 5)]);
  });

  test(
    'async callback inputs survive suspension without retaining native buffers',
    () async {
      final consumer = AsyncConsumer();
      await sendBuffersAsync(consumer: consumer);
      final before = liveBytes();
      for (var i = 0; i < 20; i++) {
        expect(await sendBuffersAsync(consumer: consumer), 96);
      }
      expect(liveBytes(), before);
      expect(consumer.saved, [List.filled(64, 4), List.filled(32, 5)]);
    },
  );

  test('callback decode failure releases later inputs too', () {
    final consumer = Consumer();
    final handle = FfiConverterCallbackInterfaceBufferConsumer.lower(consumer);
    final status = calloc<RustCallStatus>();
    final result = calloc<Uint64>();
    try {
      final before = liveBytes();
      final malformed = toRustBuffer(Uint8List.fromList([0, 0, 1, 0]));
      final later = FfiConverterUint8List.lower(Uint8List(32));
      bufferConsumerConsume(handle.address, malformed, later, result, status);
      expect(consumer.saved, isEmpty);
      expect(
        () => checkCallStatus(NullRustCallStatusErrorHandler(), status),
        throwsA(isA<UniffiInternalError>()),
      );
      expect(liveBytes(), before);
    } finally {
      bufferConsumerFreeCallback(handle.address);
      calloc.free(status);
      calloc.free(result);
    }
  });
}
