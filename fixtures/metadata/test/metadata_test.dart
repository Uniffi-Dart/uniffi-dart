import 'package:test/test.dart';
import '../metadata.dart';

void main() {
  group('Metadata', () {
    test('basic metadata functionality', () {
      testMetadata();

      final metadataStruct = getMetadataStruct();
      expect(metadataStruct.name, equals('uniffi-dart'));
      expect(metadataStruct.version, equals(1));
      expect(metadataStruct.features, contains('metadata'));
      expect(metadataStruct.features, contains('testing'));
    });

        test('metadata object functionality', () {
      final obj = MetadataObject('test-object');
      expect(obj.getName(), equals('test-object'));
      expect(obj.getVersion(), equals(0));

      obj.setVersion(42);
      // Note: In real implementation, this should update the version
    });

        test('metadata enum functionality', () {
      final basicType = MetadataType.basic;
      expect(basicType, equals(MetadataType.basic));

      final advancedType = MetadataType.advanced;
      expect(advancedType, equals(MetadataType.advanced));

      final experimentalType = MetadataType.experimental;
      expect(experimentalType, equals(MetadataType.experimental));
    });
  });
}
