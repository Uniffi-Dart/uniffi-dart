import 'package:test/test.dart';
import '../external_types_provider.dart';

void main() {
  group('External Types Provider Tests', () {
    test('ExternalRecord creation and manipulation', () {
      final record = createExternalRecord("test", 42);
      expect(record.name, "test");
      expect(record.value, 42);
      expect(record.isActive, true);

      final stringRep = externalRecordToString(record);
      expect(stringRep, contains("test"));
      expect(stringRep, contains("42"));
      expect(stringRep, contains("true"));
    });

    test('ExternalEnum handling', () {
      final firstEnum = getExternalEnum(0);
      final secondEnum = getExternalEnum(1);
      final thirdEnum = getExternalEnum(2);

      expect(externalEnumToInt(firstEnum), 0);
      expect(externalEnumToInt(secondEnum), 1);
      expect(externalEnumToInt(thirdEnum), 2);
    });

    test('ExternalObject functionality', () {
      final obj = createExternalObject("TestObject");
      expect(obj.getName(), "TestObject");

      obj.setName("ModifiedObject");
      expect(obj.getName(), "ModifiedObject");

      expect(obj.calculate(5), 11); // 5 * 2 + 1 = 11

      final objName = externalObjectGetName(obj);
      expect(objName, "ModifiedObject");
    });

    test('ExternalObject direct instantiation', () {
      final obj = ExternalObject("DirectObject");
      expect(obj.getName(), "DirectObject");
      expect(obj.calculate(10), 21); // 10 * 2 + 1 = 21
    });
  });
}
