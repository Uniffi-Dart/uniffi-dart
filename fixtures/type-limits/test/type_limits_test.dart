import 'dart:typed_data';
import 'package:test/test.dart';
import '../type_limits.dart';

void main() {
  group('Type Limits Tests', () {
    group('Signed Integer Lower Bounds', () {
      test('i8 bounds', () {
        // Test valid lower bound
        expect(takeI8(-128), -128); // -2^7

        // Test invalid lower bound
        expect(() => takeI8(-129), throwsArgumentError);
      });

      test('i16 bounds', () {
        // Test valid lower bound
        expect(takeI16(-32768), -32768); // -2^15

        // Test invalid lower bound
        expect(() => takeI16(-32769), throwsArgumentError);
      });

      test('i32 bounds', () {
        // Test valid lower bound
        expect(takeI32(-2147483648), -2147483648); // -2^31

        // Test invalid lower bound (will be caught by Dart before reaching Rust)
        expect(() => takeI32(-2147483649), throwsArgumentError);
      });

      test('i64 bounds', () {
        // Test valid lower bound
        final minI64 =
            -9223372036854775808; // -2^63 (this is within Dart's range)
        expect(takeI64(minI64), minI64);

        // Test values that would overflow are caught by Dart
        // We can't easily test overflow in Dart as it handles large integers differently
      });
    });

    group('Signed Integer Upper Bounds', () {
      test('i8 bounds', () {
        // Test valid upper bound
        expect(takeI8(127), 127); // 2^7 - 1

        // Test invalid upper bound
        expect(() => takeI8(128), throwsArgumentError);
      });

      test('i16 bounds', () {
        // Test valid upper bound
        expect(takeI16(32767), 32767); // 2^15 - 1

        // Test invalid upper bound
        expect(() => takeI16(32768), throwsArgumentError);
      });

      test('i32 bounds', () {
        // Test valid upper bound
        expect(takeI32(2147483647), 2147483647); // 2^31 - 1

        // Test invalid upper bound
        expect(() => takeI32(2147483648), throwsArgumentError);
      });

      test('i64 bounds', () {
        // Test valid upper bound
        final maxI64 =
            9223372036854775807; // 2^63 - 1 (this is within Dart's range)
        expect(takeI64(maxI64), maxI64);

        // Test values that would overflow are caught by Dart
        // We can't easily test overflow in Dart as it handles large integers differently
      });
    });

    group('Unsigned Integer Lower Bounds', () {
      test('u8 bounds', () {
        // Test valid lower bound
        expect(takeU8(0), 0);

        // Test invalid lower bound
        expect(() => takeU8(-1), throwsArgumentError);
      });

      test('u16 bounds', () {
        // Test valid lower bound
        expect(takeU16(0), 0);

        // Test invalid lower bound
        expect(() => takeU16(-1), throwsArgumentError);
      });

      test('u32 bounds', () {
        // Test valid lower bound
        expect(takeU32(0), 0);

        // Test invalid lower bound
        expect(() => takeU32(-1), throwsArgumentError);
      });

      test('u64 bounds', () {
        // Test valid lower bound
        expect(takeU64(0), 0);

        // Test invalid lower bound
        expect(() => takeU64(-1), throwsArgumentError);
      });
    });

    group('Unsigned Integer Upper Bounds', () {
      test('u8 bounds', () {
        // Test valid upper bound
        expect(takeU8(255), 255); // 2^8 - 1

        // Test invalid upper bound
        expect(() => takeU8(256), throwsArgumentError);
      });

      test('u16 bounds', () {
        // Test valid upper bound
        expect(takeU16(65535), 65535); // 2^16 - 1

        // Test invalid upper bound
        expect(() => takeU16(65536), throwsArgumentError);
      });

      test('u32 bounds', () {
        // Test valid upper bound
        expect(takeU32(4294967295), 4294967295); // 2^32 - 1

        // Test invalid upper bound
        expect(() => takeU32(4294967296), throwsArgumentError);
      });

      test('u64 bounds', () {
        // Test valid upper bound - use BigInt for large values
        final maxU64 = BigInt.parse('18446744073709551615'); // 2^64 - 1
        expect(takeU64(maxU64), maxU64);

        // Test values that would overflow are caught by Dart
        // We can't easily test overflow in Dart as it handles large integers differently
      });
    });

    group('Float Tests', () {
      test('f32 normal values', () {
        expect(takeF32(0.0), 0.0);
        expect(takeF32(1.0), 1.0);
        expect(takeF32(-1.0), -1.0);
        expect(takeF32(123.456), closeTo(123.456, 0.001));
      });

      test('f64 normal values', () {
        expect(takeF64(0.0), 0.0);
        expect(takeF64(1.0), 1.0);
        expect(takeF64(-1.0), -1.0);
        expect(takeF64(123.456789), closeTo(123.456789, 0.000001));
      });

      test('f32 special values', () {
        expect(takeF32(double.infinity), double.infinity);
        expect(takeF32(double.negativeInfinity), double.negativeInfinity);
        expect(takeF32(double.nan).isNaN, true);
      });

      test('f64 special values', () {
        expect(takeF64(double.infinity), double.infinity);
        expect(takeF64(double.negativeInfinity), double.negativeInfinity);
        expect(takeF64(double.nan).isNaN, true);
      });
    });

    group('String Tests', () {
      test('valid strings', () {
        expect(takeString(''), '');
        expect(takeString('hello'), 'hello');
        expect(takeString('愛'), '愛');
        expect(takeString('💖'), '💖');
      });

      test('unicode strings', () {
        expect(takeString('Hello, 世界!'), 'Hello, 世界!');
        expect(takeString('🌍🌎🌏'), '🌍🌎🌏');
      });
    });

    group('Bytes Tests', () {
      test('valid bytes', () {
        expect(takeBytes(Uint8List.fromList([])), Uint8List.fromList([]));
        expect(
          takeBytes(Uint8List.fromList([1, 2, 3])),
          Uint8List.fromList([1, 2, 3]),
        );
        expect(
          takeBytes(Uint8List.fromList([0, 255])),
          Uint8List.fromList([0, 255]),
        );
      });

      test('utf8 encoded bytes', () {
        final utf8Bytes = Uint8List.fromList('愛'.codeUnits);
        expect(takeBytes(utf8Bytes), utf8Bytes);

        final emojiBytes = Uint8List.fromList('💖'.codeUnits);
        expect(takeBytes(emojiBytes), emojiBytes);
      });

      test('binary data', () {
        final binaryData = Uint8List.fromList([
          0x00,
          0x01,
          0x02,
          0xFF,
          0xFE,
          0xFD,
        ]);
        expect(takeBytes(binaryData), binaryData);
      });
    });

    group('Large Number Tests', () {
      test('large valid numbers', () {
        expect(takeI8(100), 100);
        expect(takeI16(10000), 10000);
        expect(takeI32(1000000000), 1000000000);
        expect(takeI64(1000000000000000000), 1000000000000000000);

        expect(takeU8(100), 100);
        expect(takeU16(10000), 10000);
        expect(takeU32(1000000000), 1000000000);
        expect(takeU64(1000000000000000000), 1000000000000000000);
      });

      test('large invalid numbers', () {
        expect(() => takeI8(1000), throwsArgumentError);
        expect(() => takeI16(100000), throwsArgumentError);
        expect(() => takeI32(10000000000), throwsArgumentError);

        expect(() => takeU8(1000), throwsArgumentError);
        expect(() => takeU16(100000), throwsArgumentError);
        expect(() => takeU32(10000000000), throwsArgumentError);
      });
    });

    group('Zero and Boundary Values', () {
      test('zero values', () {
        expect(takeI8(0), 0);
        expect(takeI16(0), 0);
        expect(takeI32(0), 0);
        expect(takeI64(0), 0);
        expect(takeU8(0), 0);
        expect(takeU16(0), 0);
        expect(takeU32(0), 0);
        expect(takeU64(0), 0);
        expect(takeF32(0.0), 0.0);
        expect(takeF64(0.0), 0.0);
      });

      test('negative zero for floats', () {
        expect(takeF32(-0.0), -0.0);
        expect(takeF64(-0.0), -0.0);
      });
    });
  });
}
