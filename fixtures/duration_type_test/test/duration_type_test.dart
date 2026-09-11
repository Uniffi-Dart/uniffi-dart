import 'dart:typed_data';

import 'package:test/test.dart';

import '../duration_type_test.dart';

void main() {
  final negativeDurations = [
    Duration(microseconds: -1),
    Duration(seconds: -1),
    Duration(seconds: -1, microseconds: -1),
  ];

  test('negative duration serialization rejects before changing storage', () {
    for (final value in negativeDurations) {
      final storage = Uint8List(12)..fillRange(0, 12, 0xa5);
      expect(
        () => FfiConverterDuration.write(value, storage),
        throwsA(
          isA<ArgumentError>().having(
            (error) => error.invalidValue,
            'invalidValue',
            value,
          ),
        ),
      );
      expect(storage, everyElement(0xa5));
      expect(
        () => FfiConverterDuration.allocationSize(value),
        throwsArgumentError,
      );
      expect(() => FfiConverterDuration.lower(value), throwsArgumentError);
    }
  });

  test('negative duration arguments are rejected', () {
    for (final value in negativeDurations) {
      expect(() => getSeconds(duration: value), throwsArgumentError);
    }
  });

  test('negative durations are rejected inside compound values', () {
    for (final value in negativeDurations) {
      expect(
        () => echoDurationRecord(value: DurationRecord(value: value)),
        throwsArgumentError,
      );
      expect(() => echoOptionalDuration(value: value), throwsArgumentError);
      expect(
        () => echoDurationSequence(values: [Duration(seconds: 1), value]),
        throwsArgumentError,
      );
    }
  });

  test('zero duration roundtrips', () {
    expect(getSeconds(duration: Duration.zero), 0);
    expect(getNanos(duration: Duration.zero), 0);
  });

  test('nonnegative compound durations roundtrip', () {
    final value = Duration(seconds: 2, microseconds: 3);
    expect(
      echoDurationRecord(value: DurationRecord(value: value)).value,
      value,
    );
    expect(echoOptionalDuration(value: null), isNull);
    expect(echoOptionalDuration(value: Duration.zero), Duration.zero);
    expect(echoOptionalDuration(value: value), value);
    expect(echoDurationSequence(values: [Duration.zero, value]), [
      Duration.zero,
      value,
    ]);
  });

  test('rust return value seconds check', () {
    final duration = makeDuration(seconds: 5, nanos: 0);

    expect(duration.inSeconds, 5);
    expect(getSeconds(duration: duration), 5);
    expect(getNanos(duration: duration), 0);
  });

  test('seconds data check from dart', () {
    final duration = Duration(seconds: 10);
    expect(getSeconds(duration: duration), 10);
    expect(getNanos(duration: duration), 0);
  });

  test('check nanos/micros', () {
    final duration = makeDuration(seconds: 0, nanos: 3000);
    expect(duration.inSeconds, 0);
    expect(duration.inMicroseconds, 3);
    expect(getSeconds(duration: duration), 0);
    expect(getNanos(duration: duration), 3000);
  });

  test('check large values', () {
    final duration = makeDuration(seconds: 123456789, nanos: 3000000);
    expect(duration.inSeconds, 123456789);
    expect(duration.inMicroseconds, 123456789003000);
    expect(getSeconds(duration: duration), 123456789);
    expect(getNanos(duration: duration), 3000000);
  });
}
