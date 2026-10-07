import 'dart:typed_data';

import 'package:test/test.dart';
import '../proc_macro.dart';

// Mock callback interface implementations
class DartTestCallbackInterface implements TestCallbackInterface {
  @override
  void doNothing() {
    // Does nothing
  }

  @override
  int add(int a, int b) {
    return a + b;
  }

  @override
  int optional(int? a) {
    return a ?? 0;
  }

  @override
  Uint8List withBytes(RecordWithBytes rwb) => Uint8List.fromList(rwb.someBytes);

  @override
  int tryParseInt(String value) {
    if (value == 'force-unexpected-error') {
      throw StateError('forced unexpected error');
    }
    final parsed = int.tryParse(value);
    if (parsed == null) {
      throw InvalidInputBasicException();
    }
    return parsed;
  }

  @override
  int callbackHandler(Object h) {
    return h.isHeavy() == MaybeBool.uncertain ? 42 : 0;
  }

  @override
  OtherCallbackInterface getOtherCallbackInterface() {
    return DartOtherCallbackInterface();
  }
}

class DartOtherCallbackInterface implements OtherCallbackInterface {
  @override
  int multiply(int a, int b) {
    return a * b;
  }
}

void main() {
  ensureInitialized();

  group('Proc-Macro No Implicit Prelude', () {
    test('basic records without prelude', () {
      final one = makeOne(inner: 42);
      expect(one.inner, equals(42));
      expect(oneInnerByRef(one: one), equals(42));

      final two = Two(a: 'hello');
      expect(takeTwo(two: two), equals('hello'));
    });

    test('nested and complex records', () {
      final nested = NestedRecord(userTypeInBuiltinGeneric: Two(a: 'nested'));
      expect(nested.userTypeInBuiltinGeneric?.a, equals('nested'));

      final recordWithBytes = makeRecordWithBytes();
      expect(recordWithBytes.someBytes, equals([0, 1, 2, 3, 4]));

      final extractedBytes = takeRecordWithBytes(rwb: recordWithBytes);
      expect(extractedBytes, equals([0, 1, 2, 3, 4]));
    });

    test('objects without prelude', () {
      final obj = Object();
      expect(obj, isNotNull);

      final namedObj = Object.namedCtor(arg: 123);
      expect(namedObj, isNotNull);

      expect(obj.isHeavy(), equals(MaybeBool.uncertain));
      expect(obj.isOtherHeavy(other: namedObj), equals(MaybeBool.uncertain));

      obj.dispose();
      namedObj.dispose();
    });

    test('enums without prelude', () {
      expect(enumIdentity(value: MaybeBool.true_), equals(MaybeBool.true_));
      expect(enumIdentity(value: MaybeBool.false_), equals(MaybeBool.false_));
      expect(
        enumIdentity(value: MaybeBool.uncertain),
        equals(MaybeBool.uncertain),
      );

      final mixedEnum = getMixedEnum(v: StringMixedEnum('test'));
      expect(mixedEnum, isA<StringMixedEnum>());

      final defaultMixed = getMixedEnum(v: null);
      expect(defaultMixed, isA<IntMixedEnum>());
      expect((defaultMixed as IntMixedEnum).v0, equals(1));
    });

    test('errors without prelude', () {
      expect(() => alwaysFails(), throwsA(isA<OsExceptionBasicException>()));

      final obj = Object();
      final result = obj.takeError(e: InvalidInputBasicException());
      expect(result, equals(42));
      obj.dispose();
    });

    test('hashmaps without prelude', () {
      final hashMap = makeHashmap(k: 1, v: 100);
      expect(hashMap, isA<Map<int, BigInt>>());

      final returned = returnHashmap(h: hashMap);
      expect(returned[1], equals(BigInt.from(100)));
    });

    test('traits without prelude', () {
      final obj = Object();
      final trait = obj.getTrait(inc: null);
      expect(trait, isNotNull);

      final result = concatStringsByRef(t: trait, a: 'hello', b: 'world');
      expect(result, equals('helloworld'));

      final traitWithForeign = obj.getTraitWithForeign(inc: null);
      expect(traitWithForeign, isNotNull);
      expect(traitWithForeign.name(), equals('RustTraitImpl'));

      obj.dispose();
      trait.dispose();
    });

    test('callback interfaces without prelude', () {
      final callback = DartTestCallbackInterface();

      expect(() => callCallbackInterface(cb: callback), returnsNormally);
    });

    test('UDL integration with proc-macro types', () {
      final one = getOne(one: One(inner: 123));
      expect(one.inner, equals(123));

      final defaultOne = getOne(one: null);
      expect(defaultOne.inner, equals(0));

      final bool_ = getBool(b: MaybeBool.true_);
      expect(bool_, equals(MaybeBool.true_));

      final defaultBool = getBool(b: null);
      expect(defaultBool, equals(MaybeBool.uncertain));

      final obj = getObject(o: null);
      expect(obj, isNotNull);

      final externals = getExternals(e: null);
      expect(externals.one, isNull);
      expect(externals.bool, isNull);

      obj.dispose();
    });

    test('custom names', () {
      final renamed = Renamed();
      expect(renamed, isNotNull);
      expect(renamed.func(), equals(true));

      expect(renameTest(), equals(true));

      renamed.dispose();
    });

    test('explicit values for defaultable APIs', () {
      expect(doubleWithDefault(), equals(42));
      expect(doubleWithDefault(num: 21), equals(42));
      expect(doubleWithDefault(num: 10), equals(20));

      final objWithDefaults = ObjectWithDefaults(num: 30);
      expect(objWithDefaults.addToNum(), equals(42));
      expect(objWithDefaults.addToNum(other: 12), equals(42));
      expect(objWithDefaults.addToNum(other: 5), equals(35));
      objWithDefaults.dispose();

      final defaultObjWithDefaults = ObjectWithDefaults();
      expect(defaultObjWithDefaults.addToNum(other: 12), equals(42));
      defaultObjWithDefaults.dispose();
    });

    test('string operations without prelude', () {
      final result = join(parts: ['hello', 'world'], sep: ' ');
      expect(result, equals('hello world'));

      final zero = makeZero();
      expect(zero.inner, equals('ZERO'));
    });

    test('flat errors', () {
      final obj = Object();

      expect(() => obj.doStuff(times: 0), throwsA(FlatException.invalidInput));
      expect(() => obj.doStuff(times: 1), returnsNormally);
      obj.dispose();
    });

    test('comprehensive proc-macro functionality', () {
      final one = One(inner: 999);
      expect(one.inner, equals(999));

      final maybeBool = MaybeBool.true_;
      expect(maybeBool, equals(MaybeBool.true_));

      final obj = Object();
      expect(obj, isNotNull);

      final recordBytes = RecordWithBytes(
        someBytes: Uint8List.fromList([1, 2, 3]),
      );
      expect(recordBytes.someBytes, equals([1, 2, 3]));

      final zero = Zero(inner: 'test');
      expect(zero.inner, equals('test'));

      obj.dispose();
    });
  });
}
