import 'dart:typed_data';

import 'package:test/test.dart';

import '../conversion_temporaries.dart';

class TextCallback implements OptionalText {
  TextCallback(this.value);
  final String? value;
  @override
  String? text() => value;
}

void main() {
  ensureInitialized();

  test(
    'nested enum writes avoid native temporaries and respect buffer offsets',
    () {
      final value = PaintColored(shade: Shade.dark, number: 42);
      final storage = Uint8List(20)..fillRange(0, 20, 0xA5);
      final view = Uint8List.sublistView(storage, 4, 16);
      final before = liveBytes();
      for (var i = 0; i < 100; i++) {
        expect(FfiConverterColored.write(value, view), 12);
      }
      expect(liveBytes(), before);
      expect(view, [0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 42]);
      expect(storage.sublist(0, 4), List.filled(4, 0xA5));
      expect(storage.sublist(16), List.filled(4, 0xA5));
    },
  );

  test(
    'nested enum reads avoid native temporaries and preserve following fields',
    () {
      final storage = Uint8List.fromList([
        99,
        99,
        0,
        0,
        0,
        1,
        0,
        0,
        0,
        2,
        0,
        0,
        0,
        42,
      ]);
      final view = Uint8List.sublistView(storage, 2);
      final before = liveBytes();
      for (var i = 0; i < 100; i++) {
        final decoded = FfiConverterColored.read(view);
        expect(decoded.bytesRead, 12);
        final value = decoded.value as PaintColored;
        expect(value.shade, Shade.dark);
        expect(value.number, 42);
      }
      expect(liveBytes(), before);
    },
  );

  test('nested enum serialization is accepted by Rust', () {
    expect(
      checkColored(value: PaintColored(shade: Shade.dark, number: 42)),
      isTrue,
    );
  });

  for (final present in [false, true]) {
    test('optional callback transfers its only buffer, present=$present', () {
      final callback = TextCallback(present ? 'x' * 64 : null);
      expect(checkOptionalText(callback: callback, present: present), isTrue);
      final before = liveBytes();
      for (var i = 0; i < 100; i++) {
        expect(checkOptionalText(callback: callback, present: present), isTrue);
      }
      expect(liveBytes(), before);
    });
  }
}
