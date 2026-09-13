import 'dart:typed_data';

import 'package:test/test.dart';

import '../time_types.dart';

Uint8List timestampBytes(int seconds, int nanos) {
  final bytes = ByteData(12)
    ..setInt64(0, seconds)
    ..setUint32(8, nanos);
  return bytes.buffer.asUint8List();
}

class EchoTime extends TimeSource {
  int calls = 0;
  @override
  DateTime echo(DateTime value) {
    calls++;
    expect(value.isUtc, isTrue);
    return value.add(const Duration(microseconds: 1));
  }
}

void main() {
  final epoch = DateTime.fromMicrosecondsSinceEpoch(0, isUtc: true);

  test('epoch and microsecond boundaries round trip through Rust', () {
    for (final micros in [
      0,
      1,
      999999,
      1000000,
      1000001,
      -1000000,
      -1000001,
      -1999999,
      -2000000,
      1700000000123456,
      -1700000000123456,
    ]) {
      final value = DateTime.fromMicrosecondsSinceEpoch(micros, isUtc: true);
      final returned = returnTimestamp(a: value);
      expect(returned.microsecondsSinceEpoch, micros);
      expect(returned.isUtc, isTrue);
    }
  });

  test('local input preserves its instant and lifts as UTC', () {
    final local = DateTime(2020, 6, 7, 8, 9, 10, 123, 456);
    expect(local.isUtc, isFalse);
    final returned = returnTimestamp(a: local);
    expect(returned.isUtc, isTrue);
    expect(returned.microsecondsSinceEpoch, local.microsecondsSinceEpoch);
  });

  test('Rust nanoseconds truncate toward epoch on both sides', () {
    for (final seconds in [2, -2]) {
      final value = timestampFromParts(seconds: seconds, nanos: 123456789);
      expect(value.microsecondsSinceEpoch, seconds > 0 ? 2123456 : -2123456);
      expect(timestampNanos(value: value), 123456000);
    }
    expect(
      timestampFromParts(seconds: 0, nanos: 999).microsecondsSinceEpoch,
      0,
    );
    expect(
      timestampFromParts(seconds: 0, nanos: 1000).microsecondsSinceEpoch,
      1,
    );
    expect(
      timestampFromParts(seconds: 0, nanos: 1999).microsecondsSinceEpoch,
      1,
    );
  });

  test('current time and existing pre-epoch helper return UTC dates', () {
    expect(now().isUtc, isTrue);
    expect(getPreEpochTimestamp().microsecondsSinceEpoch, -1001000);
    expect(
      getSecondsBeforeUnixEpoch(a: epoch.subtract(const Duration(seconds: 2))),
      2,
    );
    expect(
      setSecondsBeforeUnixEpoch(seconds: 2).microsecondsSinceEpoch,
      -2000000,
    );
  });

  test('arithmetic and declared errors use timestamp arguments', () {
    final value = epoch.add(const Duration(seconds: 42));
    final later = add(a: value, b: const Duration(microseconds: 1234567));
    expect(diff(a: later, b: value), const Duration(microseconds: 1234567));
    expect(
      equal(
        a: value,
        b: returnTimestamp(a: value),
      ),
      isTrue,
    );
    expect(
      () => diff(a: value, b: later),
      throwsA(isA<ChronologicalException>()),
    );
    expect(
      returnDuration(a: const Duration(microseconds: 123)),
      const Duration(microseconds: 123),
    );
    expect(toStringTimestamp(a: value), '1970-01-01T00:00:42.000000000Z');
  });

  test('optional timestamps lower present and absent values', () {
    expect(optional(a: epoch, b: Duration.zero), isTrue);
    expect(optional(a: null, b: Duration.zero), isFalse);
    expect(optional(a: epoch, b: null), isFalse);
  });

  test('records, optional fields, lists and maps preserve timestamps', () {
    final before = epoch.subtract(const Duration(seconds: 2, microseconds: 3));
    for (final optionalValue in [null, before]) {
      final result = echoTimeBundle(
        value: TimeBundle(
          at: epoch,
          optionalAt: optionalValue,
          history: [before, epoch],
          named: {'before': before, 'epoch': epoch},
        ),
      );
      expect(result.at, epoch);
      expect(result.optionalAt, optionalValue);
      expect(result.history, [before, epoch]);
      expect(result.named, {'before': before, 'epoch': epoch});
    }
  });

  test('timestamp callback uses RustBuffer input and output', () {
    final source = EchoTime();
    final returned = callbackTimestamp(source: source, value: epoch);
    expect(returned.microsecondsSinceEpoch, 1);
    expect(source.calls, 1);
  });

  test('timestamp object constructor and methods work', () {
    final clock = TimeKeeper(value: epoch);
    try {
      expect(clock.get_(), epoch);
      expect(
        clock
            .echo(value: epoch.add(const Duration(microseconds: 1)))
            .microsecondsSinceEpoch,
        1,
      );
    } finally {
      clock.dispose();
    }
  });

  test('async timestamp return lifts to DateTime', () async {
    final value = epoch.subtract(const Duration(seconds: 2, microseconds: 3));
    expect(await returnTimestampAsync(value: value), value);
  });

  test('converter respects a byte view offset and reports consumed bytes', () {
    final backing = Uint8List(20)..fillRange(0, 20, 0xaa);
    final slice = Uint8List.sublistView(backing, 4, 16);
    final value = epoch.subtract(
      const Duration(seconds: 2, microseconds: 123456),
    );
    expect(FfiConverterTimestamp.write(value, slice), 12);
    expect(slice, timestampBytes(-2, 123456000));
    final read = FfiConverterTimestamp.read(slice);
    expect(read.value, value);
    expect(read.bytesRead, 12);
    expect(backing.take(4), everyElement(0xaa));
    expect(backing.skip(16), everyElement(0xaa));
  });

  test('DateTime endpoints are checked before integer multiplication', () {
    const maxSeconds = 8640000000000;
    for (final seconds in [-maxSeconds, maxSeconds]) {
      final decoded = FfiConverterTimestamp.read(timestampBytes(seconds, 0))
          .value;
      expect(decoded.microsecondsSinceEpoch, seconds * 1000000);
      final bytes = Uint8List(12);
      FfiConverterTimestamp.write(decoded, bytes);
      expect(bytes, timestampBytes(seconds, 0));
      expect(
        () => FfiConverterTimestamp.read(timestampBytes(seconds, 1)),
        throwsRangeError,
      );
    }
    for (final seconds in [
      -maxSeconds - 1,
      maxSeconds + 1,
      -9223372036854775808,
      9223372036854775807,
    ]) {
      expect(
        () => FfiConverterTimestamp.read(timestampBytes(seconds, 0)),
        throwsRangeError,
      );
    }
  });

  test('short output views do not write into the surrounding buffer', () {
    final backing = Uint8List(20)..fillRange(0, 20, 0xaa);
    final shortView = Uint8List.sublistView(backing, 4, 15);
    expect(
      () => FfiConverterTimestamp.write(epoch, shortView),
      throwsRangeError,
    );
    expect(backing, everyElement(0xaa));
  });

  test('malformed nanos and short byte views are rejected', () {
    for (final nanos in [1000000000, 0xffffffff]) {
      expect(
        () => FfiConverterTimestamp.read(timestampBytes(0, nanos)),
        throwsFormatException,
      );
    }
    for (final length in [0, 8, 11]) {
      final bytes = Uint8List.sublistView(Uint8List(20), 4, 4 + length);
      expect(() => FfiConverterTimestamp.read(bytes), throwsFormatException);
    }
  });

  test(
    'negative subsecond input is rejected before writing bytes or calling Rust',
    () {
      for (final micros in [-1, -500000, -999999]) {
        final value = DateTime.fromMicrosecondsSinceEpoch(micros, isUtc: true);
        final bytes = Uint8List(12)..fillRange(0, 12, 0xaa);
        expect(
          () => FfiConverterTimestamp.write(value, bytes),
          throwsRangeError,
        );
        expect(bytes, everyElement(0xaa));
        expect(() => returnTimestamp(a: value), throwsRangeError);
      }
    },
  );
}
