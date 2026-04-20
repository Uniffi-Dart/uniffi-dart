import 'package:test/test.dart';
import '../struct_default_values.dart';

void main() {
  group('StructDefaultValues', () {
    test('bookmark only nondefault set', () {
      const url = "https://mozilla.github.io/uniffi-rs";
      final bookmark = Bookmark(position: 2, url: url);

      expect(bookmark.guid, isNull);
      expect(bookmark.position, equals(2));
      expect(bookmark.url, equals(url));
      expect(bookmark.lastModified, isNull);
      expect(bookmark.title, isNull);
    });

    test('bookmark others set', () {
      const url = "https://mozilla.github.io/uniffi-rs";
      final bookmark = Bookmark(
        position: 3,
        url: url,
        guid: "c0ffee",
        title: "Test Title",
      );

      expect(bookmark.guid, equals("c0ffee"));
      expect(bookmark.position, equals(3));
      expect(bookmark.url, equals(url));
      expect(bookmark.lastModified, isNull);
      expect(bookmark.title, equals("Test Title"));
    });

    test('bookmark all fields set', () {
      const url = "https://mozilla.github.io/uniffi-rs";
      final bookmark = Bookmark(
        position: 1,
        url: url,
        guid: "deadbeef",
        lastModified: 1234567890,
        title: "Full Bookmark",
      );

      expect(bookmark.guid, equals("deadbeef"));
      expect(bookmark.position, equals(1));
      expect(bookmark.url, equals(url));
      expect(bookmark.lastModified, equals(1234567890));
      expect(bookmark.title, equals("Full Bookmark"));
    });
  });

  group('Contact (optional fields without defaults)', () {
    test('only required fields', () {
      final contact = Contact(name: "Alice", age: 30);

      expect(contact.name, equals("Alice"));
      expect(contact.age, equals(30));
      expect(contact.email, isNull);
      expect(contact.nickname, isNull);
    });

    test('one optional field set', () {
      final contact = Contact(name: "Bob", age: 25, email: "bob@example.com");

      expect(contact.name, equals("Bob"));
      expect(contact.age, equals(25));
      expect(contact.email, equals("bob@example.com"));
      expect(contact.nickname, isNull);
    });

    test('all fields set', () {
      final contact = Contact(
        name: "Carol",
        age: 40,
        email: "carol@example.com",
        nickname: "CC",
      );

      expect(contact.name, equals("Carol"));
      expect(contact.age, equals(40));
      expect(contact.email, equals("carol@example.com"));
      expect(contact.nickname, equals("CC"));
    });
  });
}
