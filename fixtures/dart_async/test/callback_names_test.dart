import 'package:test/test.dart';

import '../dart_async.dart';

class Callback implements AsyncNameCallback {
  @override
  Future<int> sum(
    int state,
    int handle,
    int callback,
    int obj,
    int result,
    int outReturn,
    int uniffiHandle,
  ) async =>
      state +
      10 * handle +
      100 * callback +
      1000 * obj +
      10000 * result +
      100000 * outReturn +
      1000000 * uniffiHandle;
}

void main() {
  test('async callback locals preserve colliding parameter values', () async {
    expect(await checkAsyncCallbackNames(callback: Callback()), 7654321);
  });
}
