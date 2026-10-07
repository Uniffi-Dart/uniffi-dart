import 'package:test/test.dart';
import '../simple_arithmetic.dart';

void main() {
  test('2 + 2 = 4', () {
    expect(add(left: 2, right: 2), 4);
  });
  test('2 * 8 = 16', () {
    expect(multiply(left: 2, right: 8), 16);
  });
  test('2 / 8 = 0', () {
    expect(divideChecked(left: 2, right: 8), 0);
  });
  test('8 / 0 = null', () {
    expect(divideChecked(left: 8, right: 0), null);
  });
  test('8 / 2 = 4', () {
    expect(divide(left: 8, right: 2), 4);
  });
  test('u8', () {
    expect(addU8(left: 2, right: 2), 4);
  });
  test('u16', () {
    expect(addU16(left: 2, right: 2), 4);
  });
  test('u64', () {
    expect(addU64(left: 2, right: 2), BigInt.from(4));
  });

  test('i8', () {
    expect(addI8(left: 2, right: 2), 4);
  });
  test('i16', () {
    expect(addI16(left: 2, right: 2), 4);
  });
  test('i32', () {
    expect(addI32(left: 2, right: 2), 4);
  });
  test('i64', () {
    expect(addI64(left: 2, right: 2), 4);
  });
  test('f32', () {
    expect(addF32(left: 2.0, right: 2.0), 4.0);
  });
  test('f64', () {
    expect(addF64(left: 2.0, right: 2.9), 4.9);
  });

  test('get back u8', () {
    expect(getBackU8(value: 4), 4);
  });
  test('get back  u16', () {
    expect(getBackU16(value: 4), 4);
  });
  test('get back u64', () {
    expect(getBackU64(value: 4), BigInt.from(4));
  });

  test('get back  i8', () {
    expect(getBackI8(value: 4), 4);
  });
  test('get back f32', () {
    expect(getBackF32(value: 4.0), 4.0);
  });
  test('get back f64', () {
    expect(getBackF64(value: 4.9), 4.9);
  });

  test('divide by zero - success case', () {
    expect(divideByZero(numerator: 10, denominator: 2), BigInt.from(5));
  });

  test('divide by zero - error case', () {
    expect(
      () => divideByZero(numerator: 10, denominator: 0),
      throwsA(isA<DivisionByZeroMathException>()),
    );
  });

  test('divide by zero - specific error type', () {
    expect(
      () => divideByZero(numerator: 10, denominator: 0),
      throwsA(isA<MathException>()),
    );
  });
}
