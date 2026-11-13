import 'package:test/test.dart';

import '../proc_macro.dart';

class Callback implements CollidingCallbackNames {
  @override
  int sum(
    int a,
    int arg0,
    int outReturn,
    int uniffiHandle,
    int obj,
    int result,
    int state,
    int handle,
    int callback,
  ) =>
      a +
      10 * arg0 +
      100 * outReturn +
      1000 * uniffiHandle +
      10000 * obj +
      100000 * result +
      1000000 * state +
      10000000 * handle +
      100000000 * callback;
}

void main() {
  test('callback locals preserve colliding parameter values', () {
    expect(checkCallbackNames(callback: Callback()), 987654321);
  });
}
