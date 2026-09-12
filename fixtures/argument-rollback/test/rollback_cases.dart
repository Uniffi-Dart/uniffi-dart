import 'dart:typed_data';

import 'package:test/test.dart';

import '../argument_rollback.dart';
import 'tracking_allocator.dart' as tracker;

class ReentrantConsumer implements Consumer {
  @override
  int accept(String payload, int small) {
    expect(() => take(payload: 'nested', small: 256), throwsArgumentError);
    return take(payload: 'ok', small: small);
  }
}

void main() {
  ensureInitialized();
  // Rust retains the callback vtable for the process lifetime. Keep it out of
  // per-test scratch cleanup so subsequent callback calls remain valid.
  final initialHandle = FfiConverterCallbackInterfaceConsumer.lower(
    ReentrantConsumer(),
  );
  consumerFreeCallback(initialHandle.address);
  tracker.calloc.retainForProcessLifetime();
  tearDown(() {
    tracker.calloc.reset();
    expect(
      liveObjects(),
      0,
      reason: 'every fixture object must have been destroyed',
    );
  });

  test(
    'earlier owned arguments are released when a later scalar is invalid',
    () {
      final before = liveBytes();
      for (var i = 0; i < 50; i++) {
        expect(() => take(payload: 'x' * 64, small: 256), throwsArgumentError);
        expect(
          () => takeVoid(payload: 'x' * 64, small: 256),
          throwsArgumentError,
        );
      }
      expect(liveBytes(), before);
      expect(take(payload: 'works', small: 1), 6);
    },
  );

  test('async entry rolls back before a future is created', () async {
    final before = liveBytes();
    await expectLater(
      takeAsync(payload: 'x' * 64, small: 256),
      throwsArgumentError,
    );
    expect(liveBytes(), before);
    expect(await takeAsync(payload: 'works', small: 1), 6);
  });

  test('failed sync and async constructors release their inputs', () async {
    final before = liveBytes();
    expect(() => Tracked(payload: 'x' * 64, small: 256), throwsArgumentError);
    await expectLater(
      Tracked.create(payload: 'x' * 64, small: 256),
      throwsArgumentError,
    );
    expect(liveBytes(), before);
    final object = Tracked(payload: '', small: 0);
    object.dispose();
  });

  test(
    'failed methods release both the receiver clone and earlier buffers',
    () async {
      final object = Tracked(payload: '', small: 0);
      try {
        final refs = object.references();
        final before = liveBytes();
        for (var i = 0; i < 20; i++) {
          expect(
            () => object.accept(payload: 'x' * 64, small: 256),
            throwsArgumentError,
          );
          await expectLater(
            object.acceptAsync(payload: 'x' * 64, small: 256),
            throwsArgumentError,
          );
        }
        expect(object.references(), refs);
        expect(liveBytes(), before);
        expect(object.accept(payload: 'ok', small: 1), 3);
      } finally {
        object.dispose();
      }
    },
  );

  test(
    'top-level and partially serialized nested object handles roll back',
    () {
      final object = Tracked(payload: '', small: 0);
      try {
        final refs = object.references();
        final before = liveBytes();
        expect(
          () => takeObject(object: object, small: 256),
          throwsArgumentError,
        );
        expect(
          () => takePacket(
            packet: Packet(objects: [object, object], small: 256),
            small: 0,
          ),
          throwsArgumentError,
        );
        expect(
          () => takePacket(
            packet: Packet(objects: [object, object], small: 0),
            small: 256,
          ),
          throwsArgumentError,
        );
        expect(object.references(), refs);
        expect(liveBytes(), before);
        expect(
          takePacket(
            packet: Packet(objects: [object, object], small: 0),
            small: 1,
          ),
          3,
        );
        expect(object.references(), refs);
      } finally {
        object.dispose();
      }
    },
  );

  test('Rust-only and Rust-backed foreign trait receivers roll back', () {
    final object = Tracked(payload: '', small: 0);
    final rust = object.asRustTrait();
    final foreign = object.asForeignTrait();
    try {
      final refs = object.references();
      final before = liveBytes();
      expect(
        () => rust.accept(payload: 'x' * 64, small: 256),
        throwsArgumentError,
      );
      expect(() => foreign.accept('x' * 64, 256), throwsArgumentError);
      expect(object.references(), refs);
      expect(liveBytes(), before);
      expect(rust.accept(payload: 'ok', small: 1), 3);
      expect(foreign.accept('ok', 1), 3);
    } finally {
      rust.dispose();
      // The abstract foreign trait has no disposal member.
      (foreign as dynamic).dispose();
      object.dispose();
    }
  });

  test('native callbacks can reenter without capturing their resources', () {
    expect(takeCallback(consumer: ReentrantConsumer(), small: 1), 3);
  });

  test('generated temporary names do not hide user arguments', () {
    expect(
      collidingNames(uniffiArguments: 'ok', uniffiArg0: 1, uniffiArg01: 2),
      5,
    );
  });

  test('direct converter callers retain ownership outside a call scope', () {
    final before = liveBytes();
    final buffer = toRustBuffer(Uint8List(64));
    expect(liveBytes(), before + 64);
    buffer.free();
    expect(liveBytes(), before);
  });

  test('failure before native entry rolls back prepared resources', () {
    final before = liveBytes();
    expect(
      () => uniffiWithArguments((scope) {
        toRustBuffer(Uint8List(64));
        return scope.call<int>(() => throw StateError('missing native symbol'));
      }),
      throwsStateError,
    );
    expect(liveBytes(), before);
  });
  for (var failAt = 0; failAt < 10; failAt++) {
    test('rollback works while allocations keep failing at $failAt', () {
      final object = Tracked(payload: '', small: 0);
      try {
        final refs = object.references();
        final before = liveBytes();
        tracker.calloc.failAfter = failAt;
        try {
          if (failAt < 9) {
            expect(
              () => object.acceptTwo(first: 'x' * 64, second: 'y' * 32),
              throwsStateError,
            );
            expect(tracker.calloc.failAfter, 0);
          } else {
            expect(object.acceptTwo(first: 'x' * 64, second: 'y' * 32), 96);
          }
        } finally {
          tracker.calloc.failAfter = null;
        }
        expect(object.references(), refs);
        expect(liveBytes(), before);
      } finally {
        object.dispose();
      }
    });
  }

  test('failed callback arguments remove their Dart handle entries', () {
    final before = callbackHandlesForTest();
    for (var i = 0; i < 20; i++) {
      expect(
        () => takeCallback(consumer: ReentrantConsumer(), small: 256),
        throwsArgumentError,
      );
    }
    expect(callbackHandlesForTest(), before);
    expect(takeCallback(consumer: ReentrantConsumer(), small: 1), 3);
    expect(callbackHandlesForTest(), before);
  });
  test('an error status still transfers arguments to Rust', () async {
    final object = Tracked(payload: '', small: 0);
    try {
      final refs = object.references();
      expect(
        () => reject(object: object, payload: 'owned'),
        throwsA(isA<Exception>()),
      );
      expect(object.references(), refs);
      await expectLater(
        rejectAsync(object: object, payload: 'owned'),
        throwsA(isA<Exception>()),
      );
      expect(object.references(), refs);
    } finally {
      object.dispose();
    }
  });
}
