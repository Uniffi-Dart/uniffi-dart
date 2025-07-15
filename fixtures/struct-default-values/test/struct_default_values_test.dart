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
}
