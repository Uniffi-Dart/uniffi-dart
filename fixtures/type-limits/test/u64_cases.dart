// Shared by package:test and the standalone JIT/AOT probe.
import 'dart:async';
import 'dart:math';
import 'dart:typed_data';

import '../type_limits.dart';

final u64Max = (BigInt.one << 64) - BigInt.one;
final intMax = (BigInt.one << 63) - BigInt.one;

void equal(Object? actual, Object? expected) {
  if (actual != expected) throw StateError('Expected $expected, got $actual');
}

void rejects<T extends Object>(void Function() action) {
  try {
    action();
  } catch (error) {
    if (error is T) return;
    rethrow;
  }
  throw StateError('Expected $T');
}

class SyncU64 implements U64Callback {
  final Object Function(BigInt) body;
  SyncU64(this.body);
  @override
  Object transform(BigInt value) => body(value);
  @override
  Object transformAlias(U64Alias value) => body(value);
  @override
  List<Object?> transformSequence(List<BigInt?> values) => [42, ...values];
}

class AsyncU64 implements AsyncU64Callback {
  final Object Function(BigInt) body;
  AsyncU64(this.body);
  @override
  Future<Object> transform(BigInt value) async => body(value);
  @override
  Future<List<Object?>> transformSequence(List<BigInt?> values) async => [
    42,
    ...values,
  ];
}

final Map<String, FutureOr<void> Function()> u64Cases = {
  'scalar boundaries and int input': () {
    final values = <BigInt>[
      BigInt.zero,
      BigInt.one,
      (BigInt.one << 32) - BigInt.one,
      (BigInt.one << 53) + BigInt.one,
      intMax,
      intMax + BigInt.one,
      u64Max - BigInt.one,
      u64Max,
    ];
    for (final value in values) {
      final BigInt result = takeU64(v: value);
      equal(result, value);
      if (value.isValidInt) equal(takeU64(v: value.toInt()), value);
    }
    equal(maxU64(), u64Max);
  },
  '1000 deterministic values cross the real native ABI': () {
    final random = Random(71);
    for (var i = 0; i < 1000; i++) {
      final value =
          (BigInt.from(random.nextInt(1 << 32)) << 32) |
          BigInt.from(random.nextInt(1 << 32));
      equal(takeU64(v: value), value);
    }
  },
  'invalid inputs never reach the Rust function': () {
    final before = u64Calls();
    for (final value in <Object>[-1, -BigInt.one, u64Max + BigInt.one]) {
      rejects<RangeError>(() => takeU64(v: value));
    }
    for (final value in <Object>[1.0, '42', true, Object()]) {
      rejects<ArgumentError>(() => takeU64(v: value));
    }
    equal(u64Calls(), before);
  },
  'checked int conversion cannot clamp': () {
    equal(BigInt.from(42).toIntChecked(), 42);
    equal(intMax.toIntChecked(), 9223372036854775807);
    equal((-intMax - BigInt.one).toIntChecked(), -9223372036854775808);
    rejects<RangeError>(() => (intMax + BigInt.one).toIntChecked());
    rejects<RangeError>(() => u64Max.toIntChecked());
    rejects<RangeError>(() => (-intMax - BigInt.two).toIntChecked());
  },
  'other integers retain int signatures': () {
    final int signed = takeI64(v: -9223372036854775808);
    final int u8 = takeU8(v: 255);
    final int u16 = takeU16(v: 65535);
    final int u32 = takeU32(v: 4294967295);
    equal(signed, -9223372036854775808);
    equal(u8, 255);
    equal(u16, 65535);
    equal(u32, 4294967295);
  },
  'byte encoding preserves all bits and respects slice offsets': () {
    final bytes = Uint8List(12);
    final slice = Uint8List.sublistView(bytes, 2, 10);
    equal(FfiConverterUInt64.write(u64Max, slice), 8);
    equal(bytes[0], 0);
    equal(bytes[10], 0);
    for (final byte in slice) {
      equal(byte, 255);
    }
    equal(FfiConverterUInt64.read(slice).value, u64Max);
    FfiConverterUInt64.write(intMax + BigInt.one, slice);
    equal(slice[0], 128);
    for (final byte in slice.skip(1)) {
      equal(byte, 0);
    }
    equal(FfiConverterUInt64.read(slice).value, intMax + BigInt.one);
  },
  'defaults and explicit overrides': () {
    equal(defaultU64(), u64Max);
    equal(defaultOptional(), u64Max);
    equal(defaultOptional(v: null), null);
    equal(defaultOptional(v: 42), BigInt.from(42));
    equal(defaultU64(v: 42), BigInt.from(42));
    final value = U64Record(value: 1, alias: 2);
    equal(value.value, BigInt.one);
    equal(value.alias, BigInt.two);
    equal(value.defaultMax, u64Max);
    equal(value.defaultZero, BigInt.zero);
    equal(value.maybe, null);
    equal(value.optionalMax, u64Max);
    final returned = recordU64(v: value);
    equal(returned.defaultMax, u64Max);
    equal(returned.alias, BigInt.two);
    equal(
      U64Record(value: 1, alias: 2, defaultMax: 3).defaultMax,
      BigInt.from(3),
    );
  },
  'optional and nested collections accept mixed inputs': () {
    equal(optionalU64(v: null), null);
    equal(optionalU64(v: 42), BigInt.from(42));
    equal(optionalU64(v: u64Max), u64Max);
    final List<BigInt?> list = sequenceU64(v: [1, null, u64Max]);
    equal(list[0], BigInt.one);
    equal(list[1], null);
    equal(list[2], u64Max);
    final Map<BigInt, List<BigInt?>> map = mapU64(
      v: {
        1: [2, null],
        u64Max: [u64Max],
      },
    );
    equal(map[BigInt.one]![0], BigInt.two);
    equal(map[BigInt.one]![1], null);
    equal(map[u64Max]!.single, u64Max);
    rejects<RangeError>(() => sequenceU64(v: [1, u64Max + BigInt.one]));
    rejects<ArgumentError>(
      () => mapU64(
        v: {
          1: ['2'],
        },
      ),
    );
    rejects<ArgumentError>(
      () => mapU64(
        v: {
          1: [1],
          BigInt.one: [2],
        },
      ),
    );
  },
  'record fields normalize at construction': () {
    final input = <Object?>[42, u64Max];
    final value = U64Record(
      value: u64Max,
      alias: 42,
      maybe: 42,
      values: input,
      mapping: {
        1: [42, u64Max],
      },
    );
    input[0] = -1;
    final BigInt stored = value.value;
    final List<BigInt?> list = value.values;
    equal(stored, u64Max);
    equal(list.first, BigInt.from(42));
    equal(value.mapping[BigInt.one]!.last, u64Max);
    equal(recordU64(v: value).maybe, BigInt.from(42));
    rejects<RangeError>(() => U64Record(value: -1, alias: 0));
    rejects<ArgumentError>(() => U64Record(value: 0, alias: 0, values: ['1']));
  },
  'enum payloads normalize and roundtrip': () {
    final single = SingleU64Value(42);
    equal(single.value, BigInt.from(42));
    equal((enumU64(v: SingleU64Value(u64Max)) as SingleU64Value).value, u64Max);
    final multiple = MultipleU64Value(
      value: u64Max,
      values: [42, null, u64Max],
    );
    equal(multiple.values.first, BigInt.from(42));
    final returned = enumU64(v: multiple) as MultipleU64Value;
    equal(returned.value, u64Max);
    equal(returned.values.last, u64Max);
    rejects<RangeError>(() => SingleU64Value(-1));
  },
  'custom aliases and object inputs': () {
    final U64Alias aliased = aliasU64(v: u64Max);
    equal(aliased, u64Max);
    equal(aliasU64(v: 42), BigInt.from(42));
    final box = U64Box(value: u64Max);
    try {
      equal(box.get_(), u64Max);
      equal(box.echo(value: 42), BigInt.from(42));
      equal(box.echo(value: u64Max), u64Max);
      rejects<RangeError>(() => box.echo(value: -1));
    } finally {
      box.dispose();
    }
  },
  'callbacks receive BigInt and can return int or BigInt': () {
    equal(callbackAlias(callback: SyncU64((v) => v), v: u64Max), u64Max);
    equal(
      callbackAlias(callback: SyncU64((v) => 42), v: u64Max),
      BigInt.from(42),
    );
    equal(
      callbackU64(
        callback: SyncU64((v) {
          equal(v, u64Max);
          return 42;
        }),
        v: u64Max,
      ),
      BigInt.from(42),
    );
    equal(callbackU64(callback: SyncU64((v) => v), v: u64Max), u64Max);
    final result = callbackSequence(
      callback: SyncU64((v) => v),
      v: [null, u64Max],
    );
    equal(result[0], BigInt.from(42));
    equal(result[1], null);
    equal(result[2], u64Max);
  },
  'invalid sync callback results report an error': () {
    for (final value in <Object>[-1, u64Max + BigInt.one, '42']) {
      try {
        callbackU64(callback: SyncU64((_) => value), v: 1);
      } catch (error) {
        if (!error.toString().contains('u64')) rethrow;
        continue;
      }
      throw StateError('Invalid callback result succeeded: $value');
    }
  },
  'async functions and callbacks preserve u64 values': () async {
    equal(await asyncU64(v: 42), BigInt.from(42));
    equal(await asyncU64(v: u64Max), u64Max);
    equal(
      await asyncCallbackU64(
        callback: AsyncU64((v) {
          equal(v, u64Max);
          return 42;
        }),
        v: u64Max,
      ),
      BigInt.from(42),
    );
    equal(
      await asyncCallbackU64(callback: AsyncU64((v) => v), v: u64Max),
      u64Max,
    );
    final result = await asyncCallbackSequence(
      callback: AsyncU64((v) => v),
      v: [null, u64Max],
    );
    equal(result.first, BigInt.from(42));
    equal(result.last, u64Max);
  },
  'invalid async callback results complete with an error': () async {
    for (final value in <Object>[-1, u64Max + BigInt.one, '42']) {
      try {
        await asyncCallbackU64(
          callback: AsyncU64((_) => value),
          v: 1,
        ).timeout(const Duration(seconds: 5));
      } on TimeoutException {
        rethrow;
      } catch (error) {
        if (!error.toString().contains('u64')) rethrow;
        continue;
      }
      throw StateError('Invalid callback result succeeded: $value');
    }
  },
};

Future<void> main() async {
  for (final entry in u64Cases.entries) {
    await entry.value();
    print('PASS: ${entry.key}');
  }
  print('${u64Cases.length} u64 cases passed');
}
