import 'package:test/test.dart';
import '../callbacks.dart'; // Adjust import to your generated code and/or callback interfaces.

class DartGetters extends ForeignGetters {
  @override
  bool getBool(bool v, bool argumentTwo) => v ^ argumentTwo;

  @override
  String getString(String v, bool arg2) {
    if (v == 'BadArgument') {
      // Throw a UniFFI-generated exception type corresponding to BadArgument
      throw SimpleException.badArgument;
    }
    if (v == 'UnexpectedError') {
      throw StateError('unexpected value');
    }
    return arg2 ? v : '1234567890123';
  }

  @override
  String? getOption(String? v, bool arg2) {
    if (v == 'BadArgument') {
      throw ReallyBadArgumentComplexException(20); // Example of a complex error
    }
    if (v == 'UnexpectedError') {
      throw StateError('unexpected value');
    }
    return arg2 ? v?.toUpperCase() : v;
  }

  @override
  List<int> getList(List<int> v, bool arg2) => arg2 ? v : <int>[];

  @override
  void getNothing(String v) {
    if (v == 'BadArgument') {
      throw SimpleException.badArgument;
    }
    if (v == 'UnexpectedError') {
      throw StateError('unexpected value');
    }
  }

  @override
  List<Item> getItems(List<Item> v) => v;

  @override
  Tag? getTag(Tag? v) => v;
}

class StoredDartStringifier extends StoredForeignStringifier {
  @override
  String fromSimpleType(int value) => 'dart: $value';

  @override
  String fromComplexType(List<double?>? values) => 'dart: $values';
}

void main() {
  ensureInitialized();
  // Initialize all VTables
  initForeignGettersVTable();
  initStoredForeignStringifierVTable();

  final callback = DartGetters();
  final rustGetters = RustGetters();
  final rustStringifier = RustStringifier(callback: StoredDartStringifier());

  test('roundtrip getBool through callback', () {
    final flag = true;
    for (final v in [true, false]) {
      final expected = callback.getBool(v, flag);
      final observed = rustGetters.getBool(
        callback: callback,
        v: v,
        argumentTwo: flag,
      );
      expect(observed, equals(expected));
    }
  });

  test('roundtrip getList through callback', () {
    final flag = true;
    for (final v in [
      [1, 2],
      [0, 1],
    ]) {
      final expected = callback.getList(v, flag);
      final observed = rustGetters.getList(
        callback: callback,
        v: v,
        arg2: flag,
      );
      expect(observed, equals(expected));
    }
  });

  test('roundtrip getString through callback', () {
    final flag = true;
    for (final v in ["Hello", "world"]) {
      final expected = callback.getString(v, flag);
      final observed = rustGetters.getString(
        callback: callback,
        v: v,
        arg2: flag,
      );
      expect(observed, equals(expected));
    }
  });

  test('roundtrip getOption through callback', () {
    final flag = true;
    for (final v in ["Some"]) {
      final expected = callback.getOption(v, flag);
      final observed = rustGetters.getOption(
        callback: callback,
        v: v,
        arg2: flag,
      );
      expect(observed, equals(expected));
    }
  });

  test('getStringOptionalCallback works', () {
    expect(
      rustGetters.getStringOptionalCallback(
        callback: callback,
        v: "1234567890123",
        arg2: false,
      ),
      equals("1234567890123"),
    );
    // Passing null as the callback
    expect(
      rustGetters.getStringOptionalCallback(
        callback: null,
        v: "1234567890123",
        arg2: false,
      ),
      isNull,
    );
  });

  test('getNothing should not throw with normal argument', () {
    // Should not throw
    rustGetters.getNothing(callback: callback, v: "1234567890123");
  });

  test('roundtrip getItems (sequence<Record>) through callback', () {
    final items = [
      Item(name: 'foo', value: 100),
      Item(name: 'bar', value: 200),
    ];
    final result = rustGetters.getItems(callback: callback, v: items);
    expect(result.length, equals(2));
    expect(result[0].name, equals('foo'));
    expect(result[0].value, equals(BigInt.from(100)));
    expect(result[1].name, equals('bar'));
    expect(result[1].value, equals(BigInt.from(200)));
  });

  test('roundtrip getTag (Optional<Record>) through callback', () {
    final tag = Tag(id: 42, label: 'test');
    final result = rustGetters.getTag(callback: callback, v: tag);
    expect(result, isNotNull);
    expect(result!.id, equals(42));
    expect(result.label, equals('test'));

    // Also test with null
    final nullResult = rustGetters.getTag(callback: callback, v: null);
    expect(nullResult, isNull);
  });

  test(
    'Rust-owned with_foreign callback can be lifted and called from Dart',
    () {
      final persister = InMemoryEventPersister().asPersister();

      persister.save('receiver-created');
      persister.save('receiver-saved');

      expect(persister.load(), equals(['receiver-created', 'receiver-saved']));
      persister.close();
    },
  );

  test('Rust-owned with_foreign callback can be passed back into Rust', () {
    final persister = InMemoryEventPersister().asPersister();

    expect(
      saveAndLoadPersister(persister: persister, event: 'payjoin-session'),
      equals(['payjoin-session']),
    );
  });

  test('getString preserves expected SimpleException.BadArgument', () {
    expect(
      () => rustGetters.getString(
        callback: callback,
        v: "BadArgument",
        arg2: true,
      ),
      throwsA(SimpleException.badArgument),
    );
  });

  test('getString maps unexpected callback exception through SimpleError', () {
    expect(
      () => rustGetters.getString(
        callback: callback,
        v: "UnexpectedError",
        arg2: false,
      ),
      throwsA(SimpleException.unexpectedError),
    );
  });

  test('getOption preserves expected ReallyBadArgumentComplexException', () {
    expect(
      () => rustGetters.getOption(
        callback: callback,
        v: "BadArgument",
        arg2: false,
      ),
      throwsA(
        predicate(
          (e) => e is ReallyBadArgumentComplexException && e.code == 20,
        ),
      ),
    );
  });

  test('getOption maps unexpected callback exception through ComplexError', () {
    expect(
      () => rustGetters.getOption(
        callback: callback,
        v: "UnexpectedError",
        arg2: false,
      ),
      throwsA(isA<UnexpectedErrorWithReasonComplexException>()),
    );
  });

  test('getNothing preserves expected SimpleException.BadArgument', () {
    expect(
      () => rustGetters.getNothing(callback: callback, v: "BadArgument"),
      throwsA(SimpleException.badArgument),
    );
  });

  test('getNothing maps unexpected callback exception through SimpleError', () {
    expect(
      () => rustGetters.getNothing(callback: callback, v: "UnexpectedError"),
      throwsA(SimpleException.unexpectedError),
    );
  });

  // test('destroy RustGetters', () {
  //   rustGetters.dispose();
  //   // No assertions; just ensure no errors are thrown.
  // });

  test('RustStringifier constructed with callback', () {
    for (final v in [1, 2]) {
      expect(rustStringifier.fromSimpleType(value: v), equals('dart: $v'));
    }
  });

  // // Clean up
  // tearDownAll(() {
  //   rustStringifier.dispose();
  // });
}
