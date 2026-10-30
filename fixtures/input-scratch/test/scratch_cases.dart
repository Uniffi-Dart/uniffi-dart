import 'dart:typed_data';

import 'package:test/test.dart';

import '../input_scratch.dart';
import 'tracking_allocator.dart' as tracker;

void main() {
  ensureInitialized();
  tearDown(tracker.calloc.reset);

  test(
    'owned and optional inputs release scratch after native consumption',
    () {
      for (var i = 0; i < 100; i++) {
        expect(ownedLen(value: Uint8List.fromList([1, 2, 3])), 3);
        expect(ownedLen(value: Uint8List(0)), 0);
        expect(optionalLen(value: 'x' * 64), 64);
        expect(optionalLen(value: ''), 0);
        expect(optionalLen(value: null), 0);
        expect(tracker.calloc.live, isEmpty);
      }
    },
  );

  test('scratch cleanup preserves the copied RustBuffer', () {
    final input = Uint8List.fromList([1, 2, 3]);
    final buffer = toRustBuffer(input);
    try {
      expect(tracker.calloc.live, isEmpty);
      input[0] = 99;
      expect(buffer.asUint8List(), [1, 2, 3]);
    } finally {
      buffer.free();
    }
    expect(tracker.calloc.live, isEmpty);
  });

  for (final optional in [false, true]) {
    for (var failAt = 0; failAt < 3; failAt++) {
      test(
        'partial allocation failure releases scratch: optional=$optional, allocation=$failAt',
        () {
          // Payload, ForeignBytes, and RustCallStatus are allocated in that order.
          tracker.calloc.failAfter = failAt;
          try {
            expect(
              () => optional
                  ? FfiConverterOptionalString.lower('x' * 64)
                  : toRustBuffer(Uint8List(64)),
              throwsStateError,
            );
          } finally {
            tracker.calloc.failAfter = null;
          }
          expect(tracker.calloc.live, isEmpty);
        },
      );
    }
  }

  test('optional serialization failure retains no native scratch', () {
    expect(() => optionalSmall(value: 256), throwsArgumentError);
    expect(tracker.calloc.live, isEmpty);
  });
}
