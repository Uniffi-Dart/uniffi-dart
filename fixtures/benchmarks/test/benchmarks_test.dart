import 'package:test/test.dart';
import '../benchmarks.dart';

class DartTestCallbackInterface implements TestCallbackInterface {
  @override
  String method(int a, int b, TestData data) {
    // Return the bar field as expected by the benchmark
    return data.bar;
  }

  @override
  void methodWithVoidReturn(int a, int b, TestData data) {
    // Intentionally does nothing - testing void return callback overhead
  }

  @override
  void methodWithNoArgsAndVoidReturn() {
    // Intentionally does nothing - testing minimal callback overhead
  }

  @override
  int runTest(TestCase testCase, int count) {
    final stopwatch = Stopwatch()..start();

    switch (testCase) {
      case TestCase.function:
        for (int i = 0; i < count; i++) {
          testFunction(
            a: 10,
            b: 100,
            data: TestData(foo: 'SomeStringData', bar: 'SomeMoreStringData'),
          );
        }
        break;
      case TestCase.voidReturn:
        for (int i = 0; i < count; i++) {
          testVoidReturn(
            a: 10,
            b: 100,
            data: TestData(foo: 'SomeStringData', bar: 'SomeMoreStringData'),
          );
        }
        break;
      case TestCase.noArgsVoidReturn:
        for (int i = 0; i < count; i++) {
          testNoArgsVoidReturn();
        }
        break;
    }

    stopwatch.stop();
    return stopwatch.elapsedMicroseconds * 1000; // Convert to nanoseconds
  }
}

void main() {
  group('Benchmarks', () {
    test('basic function benchmarking', () {
      final result = testFunction(
        a: 10,
        b: 100,
        data: TestData(foo: 'TestFoo', bar: 'TestBar'),
      );
      expect(result, equals('TestBar'));

      // Test void return
      testVoidReturn(
        a: 10,
        b: 100,
        data: TestData(foo: 'TestFoo', bar: 'TestBar'),
      );

      // Test no args void return
      testNoArgsVoidReturn();
    });

        test('callback interface benchmarking', () {
      final callback = DartTestCallbackInterface();

      // Test callback methods
      final result = callback.method(
        10,
        100,
        TestData(foo: 'TestFoo', bar: 'TestBar'),
      );
      expect(result, equals('TestBar'));

      // Test void callback
      callback.methodWithVoidReturn(
        10,
        100,
        TestData(foo: 'TestFoo', bar: 'TestBar'),
      );

      // Test no-args void callback
      callback.methodWithNoArgsAndVoidReturn();
    });

    test('performance test runner', () {
      // This test will fail until callback interface support is implemented
      // Expected: Should be able to run timed performance tests

      final callback = DartTestCallbackInterface();

      // Run small performance tests
      final functionTime = callback.runTest(TestCase.function, 10);
      expect(functionTime, greaterThan(0));

      final voidTime = callback.runTest(TestCase.voidReturn, 10);
      expect(voidTime, greaterThan(0));

      final noArgsTime = callback.runTest(TestCase.noArgsVoidReturn, 10);
      expect(noArgsTime, greaterThan(0));
    });

    test('full benchmark suite', () {
      // This test will fail until callback interface support is implemented
      // Expected: Should be able to run the full benchmark suite

      final callback = DartTestCallbackInterface();

      expect(() => runBenchmarks(languageName: 'Dart', cb: callback), returnsNormally);
    });
  });
}
