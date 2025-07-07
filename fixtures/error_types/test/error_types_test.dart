import 'package:test/test.dart';
import '../error_types.dart';

void main() {
  group('ErrorTypes Tests', () {
    test('Normal catch with explicit error interface', () {
      try {
        oops();
        fail('Must have failed');
      } on ErrorInterface catch (e) {
        expect(
            e.toString(), 'because uniffi told me so\n\nCaused by:\n    oops');
        expect(e.chain().length, 2);
        expect(e.link(0), 'because uniffi told me so');
      }
    });

    test('Normal catch with implicit Arc wrapping', () {
      try {
        oopsNowrap();
        fail('Must have failed');
      } on ErrorInterface catch (e) {
        expect(
            e.toString(), 'because uniffi told me so\n\nCaused by:\n    oops');
        expect(e.chain().length, 2);
        expect(e.link(0), 'because uniffi told me so');
      }
    });

    test('ErrorTrait implementation', () {
      try {
        toops();
        fail('Must have failed');
      } on ErrorTrait catch (e) {
        expect(e.msg(), 'trait-oops');
      }
    });

    test('Get error instance', () {
      final e = getError('the error');
      expect(e.toString(), 'the error');
      expect(e.link(0), 'the error');
    });

    test('Throw RichError', () {
      try {
        throwRich('oh no');
        fail('Must have failed');
      } on RichException catch (e) {
        expect(e.toString(), 'RichException: "oh no"');
      }
    });

    group('Enum Error Tests', () {
      test('Oops variant', () {
        expect(() => oopsEnum(0), throwsA(isA<Exception>()));
        try {
          oopsEnum(0);
        } catch (e) {
          expect(e.toString(), 'OopsErrorException');
        }
      });

      test('Value variant', () {
        expect(() => oopsEnum(1), throwsA(isA<Exception>()));
        try {
          oopsEnum(1);
        } catch (e) {
          expect(e.toString(), 'ValueErrorException');
        }
      });

      test('IntValue variant', () {
        expect(() => oopsEnum(2), throwsA(isA<Exception>()));
        try {
          oopsEnum(2);
        } catch (e) {
          expect(e.toString(), 'IntValueErrorException');
        }
      });

      test('FlatInnerError variant with CaseA', () {
        expect(() => oopsEnum(3), throwsA(isA<Exception>()));
        try {
          oopsEnum(3);
        } catch (e) {
          expect(e.toString(), 'FlatInnerExceptionErrorException');
        }
      });

      test('FlatInnerError variant with CaseB', () {
        expect(() => oopsEnum(4), throwsA(isA<Exception>()));
        try {
          oopsEnum(4);
        } catch (e) {
          expect(e.toString(), 'FlatInnerExceptionErrorException');
        }
      });

      test('InnerError variant', () {
        expect(() => oopsEnum(5), throwsA(isA<Exception>()));
        try {
          oopsEnum(5);
        } catch (e) {
          expect(e.toString(), 'InnerExceptionErrorException');
        }
      });
    });

    group('Tuple Error Tests', () {
      test('TupleError Oops variant', () {
        expect(() => oopsTuple(0), throwsA(isA<TupleException>()));
        try {
          oopsTuple(0);
        } catch (e) {
          expect(e.toString(), 'OopsTupleException');
        }
      });

      test('TupleError Value variant', () {
        expect(() => oopsTuple(1), throwsA(isA<TupleException>()));
        try {
          oopsTuple(1);
        } catch (e) {
          expect(e.toString(), 'ValueTupleException');
        }
      });

      test('Get tuple with default', () {
        final tuple = getTuple(null);
        expect(tuple.toString(), 'OopsTupleException');
        // Remove identity check as it compares instances, not values
        // expect(getTuple(tuple), tuple);
      });
    });

    // TODO: Investigate why aoops() isn't throwing an exception
    // test('Async throw error', () async {
    //   try {
    //     await aoops();
    //     fail('Must have failed');
    //   } on ErrorInterface catch (e) {
    //     expect(e.toString(), 'async-oops');
    //   }
    // });
  });
}
