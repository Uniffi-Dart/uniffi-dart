import 'package:test/test.dart';
import '../streams_ext.dart';

void main() {
  test('Simple Stream emits expected sequence', () {
    expect(
      simpleStream(),
      emitsInOrder([0, 1, 2, 3, 4, emitsDone]),
    );
  });

  test('Count Stream emits expected sequence', () {
    expect(
      countStream(),
      emitsInOrder([0, 1, 2, 3, 4, emitsDone]),
    );
  });

  // test('Alphabet Stream emits all uppercase letters', () {
  //   expect(
  //     alphabetStream(),
  //     emitsInOrder([
  //       ...('ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('')),
  //       emitsDone,
  //     ]),
  //   );
  // });

  test('Fibonacci Stream emits first 10 Fibonacci numbers', () {
    expect(
      fibonacciStream().take(10),
      emitsInOrder([...([0, 1, 1, 2, 3, 5, 8, 13, 21, 34].map(BigInt.from)), emitsDone]),
    );
  });

  test('Async Timer Stream emits incrementing numbers', () {
    expect(
      asyncTimerStream().take(5),
      emitsInOrder([...([1, 2, 3, 4, 5].map(BigInt.from)), emitsDone]),
    );
  }, timeout: Timeout(Duration(seconds: 6)));

  test('Combined Streams emits from all source streams, verify count',
      () async {
    var count = 0;
    await for (final event in combinedStreams().take(15)) {
      print(event);
      count++;
    }
    print('Stream done');
    expect(count, 10);
  });

  test('Combined Streams emits from all source streams, specifically', () {
    expect(
      combinedStreams(),
      emitsInAnyOrder([
        'Count: 0',
        'Fibonacci: 0',
        'Count: 1',
        'Fibonacci: 1',
        'Count: 2',
        'Fibonacci: 1',
        'Count: 3',
        'Fibonacci: 2',
        'Count: 4',
        'Fibonacci: 3',
        emitsDone,
      ]),
    );
  });

  test('Combined Streams emits from all source streams', () {
    expect(
      combinedStreams(),
      emitsInAnyOrder([
        predicate((String s) => s.startsWith('Count:')),
        predicate((String s) => s.startsWith('Fibonacci:')),
      ]),
    );
  });
}
