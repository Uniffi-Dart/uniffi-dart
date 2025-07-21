import 'dart:typed_data';

import 'package:test/test.dart';
import '../bytes_types.dart';

void main() {
  group('Basic Bytes Functions', () {
    test('take_bytes returns same data', () {
      final input = [1, 2, 3, 4, 5];
      final result = takeBytes(Uint8List.fromList(input));
      expect(result, equals(input));
    });

    // test('take_bytes_with_validation handles UTF-8', () {
    //   final utf8Input = 'Hello, 世界!'.codeUnits;
    //   final result = takeBytesWithValidation(utf8Input);
    //   expect(result, equals(utf8Input));
    // });

    // test('take_empty_bytes returns empty list', () {
    //   final result = takeEmptyBytes();
    //   expect(result, isEmpty);
    // });

    // test('get_test_bytes returns expected content', () {
    //   final result = getTestBytes();
    //   final expected = 'Hello, UniFFI bytes!'.codeUnits;
    //   expect(result, equals(expected));
    // });

    // test('get_binary_bytes returns binary data', () {
    //   final result = getBinaryBytes();
    //   final expected = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    //   expect(result, equals(expected));
    // });
  });

  // group('Records with Bytes', () {
  //   test('make_record_with_bytes creates correct record', () {
  //     final record = makeRecordWithBytes();
  //     expect(record.someBytes, equals([0, 1, 2, 3, 4]));
  //   });

  //   test('take_record_with_bytes extracts bytes', () {
  //     final record = RecordWithBytes(someBytes: [10, 20, 30]);
  //     final result = takeRecordWithBytes(record);
  //     expect(result, equals([10, 20, 30]));
  //   });

  //   test('create_complex_bytes_record creates complex structure', () {
  //     final record = createComplexBytesRecord();
  //     expect(record.name, equals('test_record'));
  //     expect(record.requiredBytes, equals('required_data'.codeUnits));
  //     expect(record.optionalBytes, equals('optional_data'.codeUnits));
  //     expect(record.bytesList, hasLength(3));
  //     expect(record.bytesList[0], equals('first'.codeUnits));
  //     expect(record.bytesList[1], equals('second'.codeUnits));
  //     expect(record.bytesList[2], equals([0x01, 0x02, 0x03]));
  //     expect(record.bytesMap['key1'], equals('value1'.codeUnits));
  //     expect(record.bytesMap['key2'], equals('value2'.codeUnits));
  //     expect(record.bytesMap['binary'], equals([0xFF, 0xFE, 0xFD]));
  //     expect(record.id, equals(42));
  //   });

  //   test('create_empty_bytes_record creates empty structure', () {
  //     final record = createEmptyBytesRecord();
  //     expect(record.name, equals('empty_record'));
  //     expect(record.requiredBytes, isEmpty);
  //     expect(record.optionalBytes, isNull);
  //     expect(record.bytesList, isEmpty);
  //     expect(record.bytesMap, isEmpty);
  //     expect(record.id, equals(0));
  //   });
  // });

  // group('BytesProcessor Object', () {
  //   late BytesProcessor processor;

  //   setUp(() {
  //     processor = BytesProcessor('test_processor');
  //   });

  //   test('get_name returns correct name', () {
  //     expect(processor.getName(), equals('test_processor'));
  //   });

  //   test('reverse reverses bytes correctly', () {
  //     final input = [1, 2, 3, 4, 5];
  //     final result = processor.reverse(input);
  //     expect(result, equals([5, 4, 3, 2, 1]));
  //   });

  //   test('concat concatenates byte arrays', () {
  //     final input = [
  //       [1, 2, 3],
  //       [4, 5],
  //       [6, 7, 8, 9],
  //     ];
  //     final result = processor.concat(input);
  //     expect(result, equals([1, 2, 3, 4, 5, 6, 7, 8, 9]));
  //   });

  //   test('split_bytes splits on delimiter', () {
  //     final input = [1, 2, 0, 3, 4, 0, 5];
  //     final result = processor.splitBytes(input, 0);
  //     expect(result, hasLength(3));
  //     expect(result[0], equals([1, 2]));
  //     expect(result[1], equals([3, 4]));
  //     expect(result[2], equals([5]));
  //   });

  //   test('store_bytes and retrieve_bytes work together', () {
  //     final testData = [10, 20, 30, 40];
  //     processor.storeBytes(testData);
  //     final retrieved = processor.retrieveBytes();
  //     expect(retrieved, equals(testData));
  //   });

  //   test('process_to_record creates record with reversed bytes', () {
  //     final input = [1, 2, 3, 4];
  //     final record = processor.processToRecord(input);
  //     expect(record.someBytes, equals([4, 3, 2, 1]));
  //   });

  //   test('xor_bytes applies XOR operation', () {
  //     final input = [0x0F, 0xF0, 0x55, 0xAA];
  //     final key = 0xFF;
  //     final result = processor.xorBytes(input, key);
  //     expect(result, equals([0xF0, 0x0F, 0xAA, 0x55]));
  //   });

  //   test('get_stats calculates correct statistics', () {
  //     final input = [1, 2, 3, 4, 5];
  //     final stats = processor.getStats(input);
  //     expect(stats['length'], equals(5));
  //     expect(stats['sum'], equals(15)); // 1+2+3+4+5
  //     expect(stats['max'], equals(5));
  //     expect(stats['min'], equals(1));
  //   });

  //   test('get_stats handles empty array', () {
  //     final input = <int>[];
  //     final stats = processor.getStats(input);
  //     expect(stats['length'], equals(0));
  //     expect(stats['sum'], equals(0));
  //     expect(stats['max'], equals(0));
  //     expect(stats['min'], equals(0));
  //   });
  // });

  // group('Bytes Enum', () {
  //   test('create_bytes_enum_variants creates all variants', () {
  //     final variants = createBytesEnumVariants();
  //     expect(variants, hasLength(4));

  //     expect(variants[0], isA<BytesEnum_Empty>());
  //     expect(variants[1], isA<BytesEnum_Small>());
  //     expect(variants[2], isA<BytesEnum_Large>());
  //     expect(variants[3], isA<BytesEnum_Multiple>());
  //   });

  //   test('process_bytes_enum handles Empty variant', () {
  //     final empty = BytesEnum.empty();
  //     final result = processBytesEnum(empty);
  //     expect(result, isEmpty);
  //   });

  //   test('process_bytes_enum handles Small variant', () {
  //     final small = BytesEnum.small([1, 2, 3]);
  //     final result = processBytesEnum(small);
  //     expect(result, equals([1, 2, 3]));
  //   });

  //   test('process_bytes_enum handles Large variant', () {
  //     final large = BytesEnum.large([10, 20, 30], 'metadata');
  //     final result = processBytesEnum(large);
  //     expect(result, equals([10, 20, 30]));
  //   });

  //   test('process_bytes_enum handles Multiple variant', () {
  //     final multiple = BytesEnum.multiple([
  //       [1, 2],
  //       [3, 4],
  //       [5, 6],
  //     ]);
  //     final result = processBytesEnum(multiple);
  //     expect(result, equals([1, 2, 3, 4, 5, 6]));
  //   });
  // });

  // group('Error Handling', () {
  //   test('validate_bytes_length accepts valid length', () {
  //     final input = [1, 2, 3, 4, 5];
  //     final result = validateBytesLength(input, 3);
  //     expect(result, equals(input));
  //   });

  //   test('validate_bytes_length rejects short length', () {
  //     final input = [1, 2];
  //     expect(() => validateBytesLength(input, 5), throwsA(isA<BytesError>()));
  //   });

  //   test('validate_bytes_content accepts valid content', () {
  //     final input = [1, 2, 3, 65, 66, 67]; // Valid ASCII
  //     final result = validateBytesContent(input);
  //     expect(result, equals(input));
  //   });

  //   test('validate_bytes_content rejects invalid content', () {
  //     final input = [1, 2, 200]; // 200 > 127, invalid
  //     expect(() => validateBytesContent(input), throwsA(isA<BytesError>()));
  //   });

  //   test('fallible_bytes_processing processes non-empty data', () {
  //     final input = [1, 2, 3];
  //     final result = fallibleBytesProcessing(input);
  //     expect(result, equals([2, 3, 4])); // Each byte incremented by 1
  //   });

  //   test('fallible_bytes_processing rejects empty data', () {
  //     final input = <int>[];
  //     expect(() => fallibleBytesProcessing(input), throwsA(isA<BytesError>()));
  //   });
  // });

  // group('Utility Functions', () {
  //   test('test_bytes_encodings returns various encodings', () {
  //     final encodings = testBytesEncodings();
  //     expect(encodings.containsKey('utf8'), isTrue);
  //     expect(encodings.containsKey('ascii'), isTrue);
  //     expect(encodings.containsKey('binary'), isTrue);
  //     expect(encodings.containsKey('empty'), isTrue);
  //     expect(encodings.containsKey('single'), isTrue);
  //     expect(encodings.containsKey('large'), isTrue);

  //     expect(encodings['utf8'], equals('Hello, 世界!'.codeUnits));
  //     expect(encodings['ascii'], equals('Hello, World!'.codeUnits));
  //     expect(encodings['binary'], equals([0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]));
  //     expect(encodings['empty'], isEmpty);
  //     expect(encodings['single'], equals([42]));
  //     expect(encodings['large'], hasLength(1024));
  //   });

  //   test('test_bytes_sizes returns various sizes', () {
  //     final sizes = testBytesSizes();
  //     expect(sizes.containsKey('empty'), isTrue);
  //     expect(sizes.containsKey('tiny'), isTrue);
  //     expect(sizes.containsKey('small'), isTrue);
  //     expect(sizes.containsKey('medium'), isTrue);
  //     expect(sizes.containsKey('large'), isTrue);

  //     expect(sizes['empty'], isEmpty);
  //     expect(sizes['tiny'], hasLength(1));
  //     expect(sizes['small'], hasLength(10));
  //     expect(sizes['medium'], hasLength(1000));
  //     expect(sizes['large'], hasLength(100000));
  //   });

  //   test('comprehensive_bytes_test creates complex result', () {
  //     final result = comprehensiveBytesTest();
  //     expect(result.name, equals('comprehensive_test'));
  //     expect(result.requiredBytes, isNotEmpty);
  //     expect(result.optionalBytes, isNotNull);
  //     expect(result.bytesList, hasLength(2));
  //     expect(result.bytesMap.containsKey('reversed'), isTrue);
  //     expect(result.bytesMap.containsKey('concatenated'), isTrue);
  //     expect(result.id, equals(999));
  //   });
  // });

  // group('Callback Interface', () {
  //   test('call_bytes_callback_interface calls callback methods', () {
  //     final callback = TestBytesCallback();

  //     // This should not throw
  //     expect(() => callBytesCallbackInterface(callback), returnsNormally);

  //     // Verify callback was called
  //     expect(callback.doNothingCalled, isTrue);
  //     expect(callback.processBytesCalled, isTrue);
  //     expect(callback.withBytesRecordCalled, isTrue);
  //     expect(callback.combineBytesCalled, isTrue);
  //     expect(callback.validateBytesCalled, isTrue);
  //     expect(callback.getBytesInfoCalled, isTrue);
  //   });
  // });

  // group('Edge Cases and Performance', () {
  //   test('handles very large byte arrays', () {
  //     final largeArray = List.filled(1000000, 42);
  //     final result = takeBytes(largeArray);
  //     expect(result.length, equals(1000000));
  //     expect(result.every((b) => b == 42), isTrue);
  //   });

  //   test('handles bytes with all possible values', () {
  //     final allBytes = List.generate(256, (i) => i);
  //     final result = takeBytes(allBytes);
  //     expect(result.length, equals(256));
  //     expect(result, equals(allBytes));
  //   });

  //   test('BytesProcessor handles rapid operations', () {
  //     final processor = BytesProcessor('performance_test');
  //     final testData = [1, 2, 3, 4, 5];

  //     // Perform multiple operations rapidly
  //     for (int i = 0; i < 100; i++) {
  //       final reversed = processor.reverse(testData);
  //       expect(reversed, equals([5, 4, 3, 2, 1]));
  //     }
  //   });

  //   test('complex record with deeply nested bytes', () {
  //     final complexRecord = ComplexBytesRecord(
  //       name: 'complex_test',
  //       requiredBytes: List.generate(1000, (i) => i % 256),
  //       optionalBytes: List.generate(500, (i) => (i * 2) % 256),
  //       bytesList: List.generate(
  //         10,
  //         (i) => List.generate(100, (j) => (i + j) % 256),
  //       ),
  //       bytesMap: Map.fromEntries(
  //         List.generate(
  //           5,
  //           (i) => MapEntry('key$i', List.generate(200, (j) => (i * j) % 256)),
  //         ),
  //       ),
  //       id: 12345,
  //     );

  //     final extractedBytes = takeRecordWithBytes(
  //       RecordWithBytes(someBytes: complexRecord.requiredBytes),
  //     );
  //     expect(extractedBytes.length, equals(1000));
  //   });
  // });
}

// // Test implementation of BytesCallbackInterface
// class TestBytesCallback implements BytesCallbackInterface {
//   bool doNothingCalled = false;
//   bool processBytesCalled = false;
//   bool withBytesRecordCalled = false;
//   bool combineBytesCalled = false;
//   bool validateBytesCalled = false;
//   bool getBytesInfoCalled = false;

//   @override
//   void doNothing() {
//     doNothingCalled = true;
//   }

//   @override
//   List<int> processBytes(List<int> data) {
//     processBytesCalled = true;
//     return data.reversed.toList();
//   }

//   @override
//   List<int> withBytesRecord(RecordWithBytes record) {
//     withBytesRecordCalled = true;
//     return record.someBytes;
//   }

//   @override
//   List<int> combineBytes(List<int> first, List<int> second) {
//     combineBytesCalled = true;
//     return [...first, ...second];
//   }

//   @override
//   bool validateBytes(List<int> data) {
//     validateBytesCalled = true;
//     return data.isNotEmpty;
//   }

//   @override
//   Map<String, List<int>> getBytesInfo() {
//     getBytesInfoCalled = true;
//     return {
//       'test_key': [1, 2, 3],
//       'another_key': [4, 5, 6],
//     };
//   }
// }
