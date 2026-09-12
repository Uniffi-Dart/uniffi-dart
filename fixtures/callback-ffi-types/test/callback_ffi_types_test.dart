import 'package:test/test.dart';

import '../callback_ffi_types.dart';

class Values implements AbiValues {
  @override
  Shade shade(Shade value) => value == Shade.light ? Shade.dark : Shade.light;
  @override
  Choice choice(Choice value) => NamedChoice((value as NamedChoice).value + 1);
  @override
  Duration duration(Duration value) => value + Duration(microseconds: 2);
  @override
  Count count(Count value) => value + 1;
  @override
  Ratio ratio(Ratio value) => value + 0.25;
}

class AsyncValues implements AsyncAbiValues {
  @override
  Future<Shade> shade(Shade value) async {
    await Future<void>.delayed(Duration.zero);
    return value == Shade.light ? Shade.dark : Shade.light;
  }

  @override
  Future<Duration> duration(Duration value) async {
    await Future<void>.delayed(Duration.zero);
    return value + Duration(microseconds: 2);
  }

  @override
  Future<Count> count(Count value) async {
    await Future<void>.delayed(Duration.zero);
    return value + 1;
  }
}

void main() {
  ensureInitialized();
  final callback = Values();

  test('flat enum callback arguments and returns use RustBuffer', () {
    expect(roundtripShade(callback: callback, value: Shade.light), Shade.dark);
    expect(roundtripShade(callback: callback, value: Shade.dark), Shade.light);
  });
  test('data-carrying enum callback arguments and returns use RustBuffer', () {
    final result = roundtripChoice(callback: callback, value: NamedChoice(41));
    expect((result as NamedChoice).value, 42);
  });
  test('duration callbacks preserve seconds and microseconds', () {
    expect(
      roundtripDuration(callback: callback, value: Duration.zero),
      Duration(microseconds: 2),
    );
    expect(
      roundtripDuration(
        callback: callback,
        value: Duration(microseconds: 1000001),
      ),
      Duration(microseconds: 1000003),
    );
  });
  test('custom integer callbacks use scalar output storage', () {
    expect(roundtripCount(callback: callback, value: 0), 1);
    expect(roundtripCount(callback: callback, value: 0xFFFFFFFE), 0xFFFFFFFF);
  });
  test('custom float callbacks use scalar output storage', () {
    expect(roundtripRatio(callback: callback, value: 0.125), 0.375);
  });
  test('async callbacks use the same argument representations', () async {
    expect(await checkAsyncValues(callback: AsyncValues()), isTrue);
  }, timeout: Timeout(Duration(seconds: 10)));
}
