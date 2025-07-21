import 'package:test/test.dart';
import '../proc_macro_no_implicit_prelude.dart';

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
  List<int> withBytes(RecordWithBytes rwb) {
    return rwb.someBytes;
  }

  @override
  int tryParseInt(String value) {
    if (value == 'force-unexpected-error') {
      throw BasicError.unexpectedError(reason: 'Forced error');
    }
    final parsed = int.tryParse(value);
    if (parsed == null) {
      throw BasicError.invalidInput;
    }
    return parsed;
  }

  @override
  int callbackHandler(Object h) {
    return 42;
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
  group('Proc-Macro No Implicit Prelude', () {
    test('basic records without prelude', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Records defined with #[derive(::uniffi::Record)] should work

      final one = makeOne(42);
      expect(one.inner, equals(42));
      expect(oneInnerByRef(one), equals(42));

      final two = Two(a: 'hello');
      expect(takeTwo(two), equals('hello'));
    });

    test('nested and complex records', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Nested records and records with generic types should work

      final nested = NestedRecord(userTypeInBuiltinGeneric: Two(a: 'nested'));
      expect(nested.userTypeInBuiltinGeneric?.a, equals('nested'));

      final recordWithBytes = makeRecordWithBytes();
      expect(recordWithBytes.someBytes, equals([0, 1, 2, 3, 4]));

      final extractedBytes = takeRecordWithBytes(recordWithBytes);
      expect(extractedBytes, equals([0, 1, 2, 3, 4]));
    });

    test('objects without prelude', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Objects defined with #[derive(::uniffi::Object)] should work

      final obj = Object();
      expect(obj, isNotNull);

      final namedObj = Object.namedCtor(123);
      expect(namedObj, isNotNull);

      expect(obj.isHeavy(), equals(MaybeBool.uncertain));
      expect(obj.isOtherHeavy(namedObj), equals(MaybeBool.uncertain));
    });

    test('enums without prelude', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Enums defined with #[derive(::uniffi::Enum)] should work

      expect(enumIdentity(MaybeBool.true_), equals(MaybeBool.true_));
      expect(enumIdentity(MaybeBool.false_), equals(MaybeBool.false_));
      expect(enumIdentity(MaybeBool.uncertain), equals(MaybeBool.uncertain));

      final mixedEnum = getMixedEnum(MixedEnum.string('test'));
      expect(mixedEnum, isA<MixedEnum>());

      final defaultMixed = getMixedEnum(null);
      expect(defaultMixed, equals(MixedEnum.int(1)));
    });

    test('errors without prelude', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Error enums should work and be throwable

      expect(() => alwaysFails(), throwsA(isA<BasicError>()));

      final obj = Object();
      final result = obj.takeError(BasicError.invalidInput);
      expect(result, equals(42));
    });

    test('hashmaps without prelude', () {
      // This test will fail until HashMap support is implemented
      // Expected: HashMap types with explicit ::std:: paths should work

      final hashMap = makeHashmap(1, 100);
      expect(hashMap, isA<Map<int, int>>());

      final returned = returnHashmap(hashMap);
      expect(returned[1], equals(100));
    });

    test('traits without prelude', () {
      // This test will fail until trait support is implemented
      // Expected: Trait objects defined with #[::uniffi::export] should work

      final obj = Object();
      final trait = obj.getTrait(null);
      expect(trait, isNotNull);

      final result = concatStringsByRef(trait, 'hello', 'world');
      expect(result, equals('helloworld'));

      final traitWithForeign = obj.getTraitWithForeign(null);
      expect(traitWithForeign, isNotNull);
    });

    test(
      'callback interfaces without prelude',
      () {
        // This test will fail until callback interface support is implemented
        // Expected: Callback interfaces should work with explicit type paths

        final callback = DartTestCallbackInterface();

        // This would test the full callback interface functionality
        // callCallbackInterface(callback);
      },
      skip: 'Requires callback interface implementation',
    );

    test('UDL integration with proc-macro types', () {
      // This test will fail until UDL typedef support is implemented
      // Expected: UDL functions should work with types defined via proc-macros

      final one = getOne(One(inner: 123));
      expect(one.inner, equals(123));

      final defaultOne = getOne(null);
      expect(defaultOne.inner, equals(0));

      final bool_ = getBool(MaybeBool.true_);
      expect(bool_, equals(MaybeBool.true_));

      final defaultBool = getBool(null);
      expect(defaultBool, equals(MaybeBool.uncertain));

      final obj = getObject(null);
      expect(obj, isNotNull);

      final externals = getExternals(null);
      expect(externals, isNotNull);
    });

    test('custom names and defaults', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Custom names and default values should work

      final renamed = Renamed();
      expect(renamed, isNotNull);
      expect(renamed.func(), equals(true));

      expect(renameTest(), equals(true));

      // Test function with defaults
      expect(doubleWithDefault(), equals(42)); // uses default 21
      expect(doubleWithDefault(10), equals(20));

      // Test object with defaults
      final objWithDefaults = ObjectWithDefaults(); // uses default 30
      expect(objWithDefaults.addToNum(), equals(42)); // 30 + 12 (default)
      expect(objWithDefaults.addToNum(5), equals(35)); // 30 + 5
    });

    test('string operations without prelude', () {
      // This test will fail until proc-macro support is implemented
      // Expected: String operations with explicit ::std::string::String should work

      final result = join(['hello', 'world'], ' ');
      expect(result, equals('hello world'));

      final zero = makeZero();
      expect(zero.inner, equals('ZERO'));
    });

    test('flat errors', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Flat errors should work and be properly handled

      final obj = Object();

      expect(() => obj.doStuff(0), throwsA(isA<FlatError>()));
      expect(() => obj.doStuff(1), returnsNormally);
    });

    test('comprehensive proc-macro functionality', () {
      // This comprehensive test ensures that proc-macros work in no_implicit_prelude environment
      // Expected: All proc-macro features should work with explicit ::std:: imports

      // Test that basic types work
      final one = One(inner: 999);
      expect(one.inner, equals(999));

      // Test that enums work
      final maybeBool = MaybeBool.true_;
      expect(maybeBool, equals(MaybeBool.true_));

      // Test that objects work
      final obj = Object();
      expect(obj, isNotNull);

      // Test that records with bytes work
      final recordBytes = RecordWithBytes(someBytes: [1, 2, 3]);
      expect(recordBytes.someBytes, equals([1, 2, 3]));

      // Test that UDL-defined types work with proc-macro types
      final zero = Zero(inner: 'test');
      expect(zero.inner, equals('test'));
    });
  });
}
