import 'package:test/test.dart';

import '../type_limits.dart';

class Aliases implements ScalarAliases {
  @override
  I32Alias integer(I32Alias value) => value + 1;
  @override
  FloatAlias floating(FloatAlias value) => value + 0.25;
}

void main() {
  test('non-u64 custom callbacks also use scalar result storage', () {
    expect(checkScalarAliases(callback: Aliases()), isTrue);
  });
}
