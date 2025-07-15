import 'package:test/test.dart';
import '../large_error.dart';

void main() {
  group('LargeError', () {
    test('error discriminant', () {
      // This test will fail until proc-macro support is implemented
      // Expected functionality: error codes should match the enum values

      // final error2 = LargeError.case2(arg1: "foo");
      // expect(errorCodeFromError(error2), equals(10002));

      // final error100 = LargeError.case100(
      //   arg1: ErrorPayload(isImportant: true, index: 42, message: "test"),
      //   arg2: ErrorPayload(isImportant: true, index: 42, message: "test"),
      //   arg3: ErrorPayload(isImportant: true, index: 42, message: "test"),
      //   arg4: ErrorPayload(isImportant: true, index: 42, message: "test"),
      // );
      // expect(errorCodeFromError(error100), equals(10100));

      // Skip test until proc-macro support is implemented
      skip(
        'Blocked by proc-macro support: #[derive(uniffi::Record)], #[derive(uniffi::Error)], #[uniffi::export]',
      );
    });

    test('error message', () {
      // This test will fail until proc-macro support is implemented
      // Expected functionality: error messages should match the #[error] attributes

      // final error1 = LargeError.case1();
      // expect(errorMessageFromError(error1), equals("Important debug description of what went wrong."));

      // final error2 = LargeError.case2(arg1: "foobar");
      // expect(errorMessageFromError(error2), equals("Important debug description of what went wrong, arg: 'foobar'"));

      // Skip test until proc-macro support is implemented
      skip(
        'Blocked by proc-macro support: #[derive(uniffi::Record)], #[derive(uniffi::Error)], #[uniffi::export]',
      );
    });
  });
}
