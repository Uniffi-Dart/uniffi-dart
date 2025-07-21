import 'package:test/test.dart';
import '../simple_iface.dart';

void main() {
  group('SimpleIface', () {
    test('basic object creation and method calls', () {
      final obj = makeObject(9000);
      expect(obj.getInner(), equals(9000));

      final result = obj.someMethod();
      expect(result, isNull);
    });

    test('object with different values', () {
      final obj1 = makeObject(42);
      expect(obj1.getInner(), equals(42));

      final obj2 = makeObject(-100);
      expect(obj2.getInner(), equals(-100));
    });
  });
}
