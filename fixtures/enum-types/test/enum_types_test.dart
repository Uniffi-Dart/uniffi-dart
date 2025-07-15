import 'package:test/test.dart';
import '../enum_types.dart';

void main() {
  group('EnumTypes', () {
    test('basic animals', () {
      // Test basic enum values
      expect(Animal.dog.index, equals(0));
      expect(Animal.cat.index, equals(1));

      // Test enum parameter passing
      expect(getAnimal(null), equals(Animal.dog));
      expect(getAnimal(Animal.cat), equals(Animal.cat));
    });

    test('enum containers', () {
      // Test complex enums with objects and records
      final dogEnum = getAnimalEnum(Animal.dog);
      final catEnum = getAnimalEnum(Animal.cat);

      // Test that we can access nested data
      expect(dogEnum.runtimeType.toString(), contains('AnimalEnum'));
      expect(catEnum.runtimeType.toString(), contains('AnimalEnum'));

      // Test that different enum instances are different
      expect(dogEnum.toString(), isNot(equals(catEnum.toString())));
    });

    test('enum equality', () {
      // Test enum equality
      final dogEnum1 = getAnimalEnum(Animal.dog);
      final dogEnum2 = getAnimalEnum(Animal.dog);
      final catEnum = getAnimalEnum(Animal.cat);

      expect(dogEnum1.toString(), equals(dogEnum2.toString()));
      expect(dogEnum1.toString(), isNot(equals(catEnum.toString())));
    });
  });
}
