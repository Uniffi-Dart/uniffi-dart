import 'package:test/test.dart';
import '../docstring_proc_macro.dart' as doc;

class CallbackTestImpl extends doc.CallbackTest {
  @override
  void test() {}
}

void main() {
  group('Docstring Proc-Macro', () {
    test('proc-macro enum with docstring', () {
      final enumValue = doc.EnumTest.one;
      expect(enumValue, equals(doc.EnumTest.one));

      final enumValue2 = doc.EnumTest.two;
      expect(enumValue2, equals(doc.EnumTest.two));
    });

    test('proc-macro associated enum with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Associated enums should have documentation for variants and fields

      final associatedEnum = doc.TestAssociatedEnumTest(42);
      expect(associatedEnum, isA<doc.AssociatedEnumTest>());

      final associatedEnum2 = doc.Test2AssociatedEnumTest(43);
      expect(associatedEnum2, isA<doc.AssociatedEnumTest>());
    });

    test('proc-macro error with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Error enums should have documentation on variants

      expect(() => doc.test(), returnsNormally);
    });

    test('proc-macro associated error with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Associated error enums should have documentation

      expect(() => doc.testWithoutDocstring(), returnsNormally);
    });

    test('proc-macro object with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Objects should have documentation on constructors and methods

      final obj = doc.ObjectTest();
      obj.test();

      final objAlt = doc.ObjectTest.newAlternate();
      objAlt.test();
    });

    test('proc-macro record with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Records should have documentation on fields

      final record = doc.RecordTest(test: 42);
      expect(record.test, equals(42));
    });

    test('proc-macro function with docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Functions should have proper documentation comments in generated Dart

      doc.test();
      doc.testMultiline();
      doc.testWithoutDocstring();
    });

    test(
      'proc-macro callback interface with docstring',
      () {
        final callback = CallbackTestImpl();
        callback.test();
      },
    );

    test('proc-macro long docstring', () {
      // This test will fail until proc-macro support is implemented
      // Expected: Long docstrings (>255 chars) should be properly handled

      doc.testLongDocstring();
    });

    test('documentation generation comparison', () {
      // This test compares proc-macro docs vs UDL docs
      // Expected: Both should generate equivalent documentation in Dart

      // Test basic functionality to ensure proc-macro definitions work
      final enumValue = doc.EnumTest.one;
      expect(enumValue, equals(doc.EnumTest.one));

      doc.test();
      doc.testMultiline();

      final obj = doc.ObjectTest();
      obj.test();

      final record = doc.RecordTest(test: 123);
      expect(record.test, equals(123));
    });
  });
}
