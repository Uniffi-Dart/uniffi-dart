import 'package:test/test.dart';
import '../docstring.dart';

void main() {
  group('Docstring', () {
    test('function with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Functions should have proper documentation comments in generated Dart code

      // Test the basic function
      test();

      // Test the multiline function
      testMultiline();

      // Test function without docstring
      testWithoutDocstring();
    });

    test('enum with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Enums should have proper documentation comments in generated Dart code

      final enumValue = EnumTest.one;
      expect(enumValue, equals(EnumTest.one));

      final enumValue2 = EnumTest.two;
      expect(enumValue2, equals(EnumTest.two));
    });

    test('associated enum with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Associated enums should have proper documentation comments

      final associatedEnum = AssociatedEnumTest.test(code: 42);
      expect(associatedEnum, isA<AssociatedEnumTest>());

      final associatedEnum2 = AssociatedEnumTest.test2(code: 43);
      expect(associatedEnum2, isA<AssociatedEnumTest>());
    });

    test('error with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Errors should have proper documentation comments

      expect(() => test(), throwsA(isA<ErrorTest>()));
    });

    test('object with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Objects should have proper documentation comments

      final obj = ObjectTest();
      obj.test();

      final objAlt = ObjectTest.newAlternate();
      objAlt.test();
    });

    test('record with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Records should have proper documentation comments

      final record = RecordTest(test: 42);
      expect(record.test, equals(42));
    });

    test('callback with docstring', () {
      // This test will fail until docstring support is implemented
      // Expected: Callbacks should have proper documentation comments

      // This will require callback interface implementation
      // final callback = CallbackTestImpl();
      // callback.test();
    });
  });
}
