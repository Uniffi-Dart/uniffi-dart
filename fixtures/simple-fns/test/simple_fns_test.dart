import 'package:test/test.dart';
import '../simple_fns.dart';

void main() {
  group('Simple Functions Tests', () {
    test('get_string returns correct string', () {
      expect(getString(), 'String created by Rust');
    });

    test('get_int returns correct integer', () {
      expect(getInt(), 1289);
    });

    test('string_identity returns same string', () {
      expect(
        stringIdentity('String created by Dart'),
        'String created by Dart',
      );
    });

    test('byte_to_u32 converts correctly', () {
      expect(byteToU32(255), 255);
      expect(byteToU32(0), 0);
      expect(byteToU32(128), 128);
    });

    test('hash_map_identity returns same map', () {
      final testMap = {'a': 'b', 'hello': 'world'};
      expect(hashMapIdentity(testMap), testMap);
    });

    test('empty map identity', () {
      final emptyMap = <String, String>{};
      expect(hashMapIdentity(emptyMap), emptyMap);
    });

    group('MyHashSet Tests', () {
      test('new_set creates empty set', () {
        final set = newSet();
        expect(setContains(set, 'foo'), false);
        expect(setContains(set, 'bar'), false);
      });

      test('add_to_set and set_contains work together', () {
        final set = newSet();

        // Add items to set
        addToSet(set, 'foo');
        addToSet(set, 'bar');

        // Check that items are in set
        expect(setContains(set, 'foo'), true);
        expect(setContains(set, 'bar'), true);

        // Check that non-existent item is not in set
        expect(setContains(set, 'baz'), false);
      });

      test('set handles duplicate additions', () {
        final set = newSet();

        // Add same item multiple times
        addToSet(set, 'duplicate');
        addToSet(set, 'duplicate');
        addToSet(set, 'duplicate');

        // Should still only contain it once
        expect(setContains(set, 'duplicate'), true);
      });

      test('set with multiple items', () {
        final set = newSet();

        // Add multiple different items
        final items = ['apple', 'banana', 'cherry', 'date'];
        for (final item in items) {
          addToSet(set, item);
        }

        // Check all items are present
        for (final item in items) {
          expect(setContains(set, item), true);
        }

        // Check non-existent items are not present
        expect(setContains(set, 'elderberry'), false);
        expect(setContains(set, 'fig'), false);
      });
    });

    group('MyHashSet object methods', () {
      test('direct object creation and methods', () {
        final set = MyHashSet();

        // Test direct add method
        set.add('direct');
        expect(set.contains('direct'), true);
        expect(set.contains('not-there'), false);

        // Test multiple additions
        set.add('first');
        set.add('second');
        expect(set.contains('first'), true);
        expect(set.contains('second'), true);
      });

      test('object handles duplicates', () {
        final set = MyHashSet();

        set.add('same');
        set.add('same');
        set.add('same');

        expect(set.contains('same'), true);
      });
    });

    test('dummy function with optional parameter', () {
      // Test with null
      dummy(null);

      // Test with value
      dummy(42);
      dummy(-128);
      dummy(127);

      // These should not throw exceptions
      expect(() => dummy(null), returnsNormally);
      expect(() => dummy(0), returnsNormally);
    });

    group('Edge cases', () {
      test('empty string handling', () {
        expect(stringIdentity(''), '');
      });

      test('byte boundary values', () {
        expect(byteToU32(0), 0);
        expect(byteToU32(255), 255);
      });

      test('special characters in strings', () {
        expect(stringIdentity('Hello 世界! 🌍'), 'Hello 世界! 🌍');
      });

      test('set with special characters', () {
        final set = newSet();
        addToSet(set, 'Hello 世界!');
        addToSet(set, '🌍🌎🌏');

        expect(setContains(set, 'Hello 世界!'), true);
        expect(setContains(set, '🌍🌎🌏'), true);
      });
    });
  });
}
