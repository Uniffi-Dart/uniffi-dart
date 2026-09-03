import 'dart:typed_data';

import 'package:test/test.dart';
import '../proc_macro.dart';

class DartOtherCallback implements OtherCallbackInterface {
  @override
  int multiply(int a, int b) => a * b;
}

class DartTraitWithForeign implements TraitWithForeign {
  DartTraitWithForeign(this.value);

  final String value;

  @override
  String name() => value;
}

class DartTestCallback implements TestCallbackInterface {
  var didNothing = false;

  @override
  void doNothing() {
    didNothing = true;
  }

  @override
  int add(int a, int b) => a + b;

  @override
  int optional(int? a) => a ?? -1;

  @override
  Uint8List withBytes(RecordWithBytes rwb) {
    return Uint8List.fromList(rwb.someBytes.reversed.toList());
  }

  @override
  int tryParseInt(String value) {
    if (value == 'bad') {
      throw InvalidInputBasicException();
    }
    return int.parse(value);
  }

  @override
  int callbackHandler(Object o) {
    return o.isHeavy() == MaybeBool.uncertain ? 42 : -1;
  }

  @override
  OtherCallbackInterface getOtherCallbackInterface() => DartOtherCallback();
}

void main() {
  ensureInitialized();

  group('ProcMacro', () {
    test('UDL and proc-macro interoperability', () {
      final one = makeOne(inner: 42);
      expect(one.inner, 42);

      final two = Two(a: 'hello');
      expect(takeTwo(two: two), 'hello');

      expect(makeHashmap(k: 1, v: 100), {1: 100});

      final obj = Object();
      expect(obj.isHeavy(), MaybeBool.uncertain);

      final defaultObject = getObject(o: null);
      expect(defaultObject.isHeavy(), MaybeBool.uncertain);

      expect(() => alwaysFails(), throwsA(isA<OsExceptionBasicException>()));

      expect(getOne(one: null).inner, 0);
      expect(getOne(one: One(inner: 7)).inner, 7);
      expect(getBool(b: null), MaybeBool.uncertain);
      expect(getBool(b: MaybeBool.true_), MaybeBool.true_);

      final zero = makeZero();
      expect(zero.inner, 'ZERO');

      final externals = getExternals(e: null);
      expect(externals.one, isNull);
      expect(externals.bool, isNull);

      final filled = getExternals(
        e: Externals(one: One(inner: 5), bool: MaybeBool.false_),
      );
      expect(filled.one?.inner, 5);
      expect(filled.bool, MaybeBool.false_);

      final rustForeignTrait = getTraitWithForeign(t: null);
      expect(rustForeignTrait.name(), 'RustTraitImpl');

      final dartForeignTrait = DartTraitWithForeign('from-dart');
      expect(getTraitWithForeign(t: dartForeignTrait).name(), 'from-dart');

      obj.dispose();
      defaultObject.dispose();
    });

    test('callback interfaces', () {
      final callback = DartTestCallback();

      callbackDoNothing(callback: callback);
      expect(callback.didNothing, isTrue);
      expect(callbackAdd(callback: callback, a: 2, b: 3), 5);
      expect(callbackOptional(callback: callback, value: null), -1);
      expect(callbackOptional(callback: callback, value: 9), 9);
      expect(
        callbackWithBytes(
          callback: callback,
          bytes: Uint8List.fromList([1, 2, 3]),
        ),
        [3, 2, 1],
      );
      expect(callbackTryParseInt(callback: callback, value: '42'), 42);
      expect(callbackHandler(callback: callback), 42);
      expect(callbackGetOtherMultiply(callback: callback, a: 6, b: 7), 42);
    });

    test('callback interfaces preserve expected errors', () {
      final callback = DartTestCallback();

      expect(
        () => callbackTryParseInt(callback: callback, value: 'bad'),
        throwsA(isA<InvalidInputBasicException>()),
      );
    });

    test('trait objects', () {
      final obj = Object();

      final fromMethod = obj.getTrait(inc: null);
      expect(
        fromMethod.concatStrings(a: 'hello, ', b: 'trait'),
        'hello, trait',
      );

      final roundtripped = obj.getTrait(inc: fromMethod);
      expect(roundtripped.concatStrings(a: 'round', b: 'trip'), 'roundtrip');

      final fromFunction = getTrait(t: null);
      expect(fromFunction.concatStrings(a: 'top', b: 'level'), 'toplevel');

      final functionRoundtrip = getTrait(t: fromFunction);
      expect(functionRoundtrip.concatStrings(a: 'again', b: '!'), 'again!');

      obj.dispose();
      fromMethod.dispose();
      roundtripped.dispose();
      fromFunction.dispose();
      functionRoundtrip.dispose();
    });

    test('borrowed &[u8] arguments cross the boundary and do not leak', () {
      // Free function: content crosses the boundary intact.
      expect(sumBorrowedBytes(data: Uint8List.fromList([1, 2, 3])), 6);
      // Empty slice hits the (null, 0) path — must not throw or miscount.
      expect(sumBorrowedBytes(data: Uint8List.fromList([])), 0);

      // Method on an object (a distinct call site from the free function).
      final obj = Object();
      expect(obj.borrowedBytesLen(data: Uint8List.fromList([9, 8, 7, 6])), 4);
      expect(obj.borrowedBytesLen(data: Uint8List.fromList([])), 0);

      // Fallible variant: success path and error path both free the copy.
      expect(sumBorrowedBytesChecked(data: Uint8List.fromList([10, 20])), 30);
      expect(
        () => sumBorrowedBytesChecked(data: Uint8List.fromList([])),
        throwsA(isA<OsExceptionBasicException>()),
      );

      // Repeated calls exercise the per-call alloc/free: a leaked native copy
      // (the bug this fixes) would grow memory without bound; a double-free
      // would crash. 50k iterations complete cleanly when the arena frees each.
      var acc = 0;
      final payload = Uint8List.fromList(List<int>.generate(64, (i) => i & 0xff));
      for (var i = 0; i < 50000; i++) {
        acc += sumBorrowedBytes(data: payload);
      }
      expect(acc, 50000 * 2016); // sum(0..63) == 2016

      obj.dispose();
    });
  });
}
