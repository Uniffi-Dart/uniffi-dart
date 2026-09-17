import 'package:test/test.dart';

import '../trait_methods.dart';

T thrown<T>(void Function() call) {
  try {
    call();
  } catch (error) {
    return error as T;
  }
  throw StateError('Expected the Rust call to throw');
}

void main() {
  test('return-only flat errors are not lifted back into Rust', () {
    final error = thrown<FlatLegacyException>(failWithFlatLegacy);
    expect(error, FlatLegacyException.missing);
    expect(error.toString(), 'FlatLegacyException.missing');
  });
  test('Rust Display for a unit variant in a data-carrying error', () {
    final error = thrown<MessageException>(
      () => failWithMessage(message: null),
    );
    expect(
      error.toString(),
      'foreign utxo missing witness_utxo or non_witness_utxo',
    );
    expect(error.debugString(), 'MissingUtxo');
  });
  test('Rust Display preserves contextual Unicode and punctuation', () {
    const message = '雪: café \$amount \"quoted\"';
    final error = thrown<MessageException>(
      () => failWithMessage(message: message),
    );
    expect(error.toString(), 'failed to convert input: $message');
    expect(error.debugString(), contains('InputConversion'));
    expect((error as InputConversionMessageException).message, message);
    for (var i = 0; i < 100; i++) {
      expect(error.toString(), 'failed to convert input: $message');
    }
  });
  test('Dart-constructed errors use Rust Display too', () {
    expect(
      InputConversionMessageException('from Dart').toString(),
      'failed to convert input: from Dart',
    );
  });
  test('unit-only rich error Display and Debug', () {
    final error = thrown<UnitMessageException>(failWithUnitMessage);
    expect(error.toString(), 'the requested item does not exist');
    expect(error.debugString(), 'Missing');
    expect(
      DeniedUnitMessageException().toString(),
      'the operation is not permitted',
    );
  });
  test('Debug-only export retains the local toString fallback', () {
    final error = thrown<DebugOnlyException>(failWithDebugOnly);
    expect(error.toString(), 'DetailDebugOnlyException(7)');
    expect(error.debugString(), 'Detail { code: 7 }');
  });
  test('errors without exported traits retain their representation', () {
    final error = thrown<LocalMessageException>(failWithLocalMessage);
    expect(error.toString(), 'DetailLocalMessageException(8)');
  });
}
