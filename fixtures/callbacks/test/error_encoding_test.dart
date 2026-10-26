import 'package:test/test.dart';

import '../callbacks.dart';
import 'callbacks_test.dart' show DartGetters;

class ThrowingFormatter implements Exception {
  @override
  String toString() => throw StateError('formatter failed');
}

class FailingGetter extends DartGetters {
  final bool declared;
  FailingGetter(this.declared);
  @override
  String? getOption(String? value, bool arg2) {
    if (declared) throw InvalidByteComplexException(256);
    throw ThrowingFormatter();
  }
}

void main() {
  for (final declared in [false, true]) {
    test(
      'sync callback falls back after ${declared ? "declared error" : "formatter"} encoding fails',
      () {
        final getters = RustGetters();
        try {
          expect(
            () => getters.getOption(
              callback: FailingGetter(declared),
              v: null,
              arg2: false,
            ),
            throwsA(
              isA<UnexpectedErrorWithReasonComplexException>().having(
                (error) => error.reason,
                'reason',
                contains('could not be encoded'),
              ),
            ),
          );
        } finally {
          getters.dispose();
        }
      },
    );
  }
}
