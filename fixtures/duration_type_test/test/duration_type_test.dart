import 'package:test/test.dart';
import '../duration_type_test.dart';

void main() {
  test('rust return value seconds check', () {
    final duration = makeDuration(seconds: 5, nanos: 0);

    expect(duration.inSeconds, 5);
    expect(getSeconds(duration: duration), BigInt.from(5));
    expect(getNanos(duration: duration), 0);
  });

  test('seconds data check from dart', () {
    final duration = Duration(seconds: 10);
    expect(getSeconds(duration: duration), BigInt.from(10));
    expect(getNanos(duration: duration), 0);
  });

  test('check nanos/micros', () {
    final duration = makeDuration(seconds: 0, nanos: 3000);
    expect(duration.inSeconds, 0);
    expect(duration.inMicroseconds, 3);
    expect(getSeconds(duration: duration), BigInt.from(0));
    expect(getNanos(duration: duration), 3000);
  });

  test('check large values', () {
    final duration = makeDuration(seconds: 123456789, nanos: 3000000);
    expect(duration.inSeconds, 123456789);
    expect(duration.inMicroseconds, 123456789003000);
    expect(getSeconds(duration: duration), BigInt.from(123456789));
    expect(getNanos(duration: duration), 3000000);
  });
}
