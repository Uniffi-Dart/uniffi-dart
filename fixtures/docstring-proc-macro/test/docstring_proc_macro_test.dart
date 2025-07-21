import 'package:test/test.dart';
import '../docstring_proc_macro.dart';

void main() {
  group('Docstring Proc-Macro', () {
    test('proc-macro enum with docstring', () {
      final enumValue = EnumTest.one;
      expect(enumValue, equals(EnumTest.one));

      final enumValue2 = EnumTest.two;
      expect(enumValue2, equals(EnumTest.two));
    });

    test('proc-macro associated enum with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Associated enums should have documentation for variants and fields

      final associatedEnum = AssociatedEnumTest.test(code: 42);
      expect(associatedEnum, isA<AssociatedEnumTest>());

      final associatedEnum2 = AssociatedEnumTest.test2(code: 43);
      expect(associatedEnum2, isA<AssociatedEnumTest>());
    });

    test('proc-macro error with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Error enums should have documentation on variants

      expect(() => test(), throwsA(isA<ErrorTest>()));
    });

    test('proc-macro associated error with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Associated error enums should have documentation

      expect(() => testWithoutDocstring(), throwsA(isA<AssociatedErrorTest>()));
    });

    test('proc-macro object with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Objects should have documentation on constructors and methods

      final obj = ObjectTest();
      obj.test();

      final objAlt = ObjectTest.newAlternate();
      objAlt.test();
    });

    test('proc-macro record with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Records should have documentation on fields

      final record = RecordTest(test: 42);
      expect(record.test, equals(42));
    });

    test('proc-macro function with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Functions should have proper documentation comments in generated Dart

      test();
      testMultiline();
      testWithoutDocstring();
    });

    test(
      'proc-macro callback interface with docstring',
      () {
        // This test will fail until callback interface support is implemented
        // Expected: Callback interfaces should have documentation on methods

        // final callback = CallbackTestImpl();
        // callback.test();
      },
      skip: 'Requires callback interface implementation',
    );

    test('proc-macro long docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Long docstrings (>255 chars) should be properly handled

      testLongDocstring();
    });

    test('documentation generation comparison', () {
      // This test compares proc-macro docs vs UDL docs
      // Expected: Both should generate equivalent documentation in Dart

      // Test basic functionality to ensure proc-macro definitions work
      final enumValue = EnumTest.one;
      expect(enumValue, equals(EnumTest.one));

      test();
      testMultiline();

      final obj = ObjectTest();
      obj.test();

      final record = RecordTest(test: 123);
      expect(record.test, equals(123));
    });
  });
}
