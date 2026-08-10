import 'dart:typed_data';

import 'package:test/test.dart';
import '../custom_types.dart';

/// Records what Rust passes in, so the test can assert on both directions of the
/// round trip.
class UpperCaseCustomIdTransformer extends CustomIdTransformer {
  final List<CustomId> received = <CustomId>[];

  @override
  CustomId transform(CustomId id) {
    received.add(id);
    return id.toUpperCase();
  }
}

void main() {
  test('custom alias bytes and nested map helpers round trip', () {
    final response = getZenEngineResponse();

    expect(response.performance, equals('ready'));
    expect(response.result, orderedEquals([1, 2, 3]));
    expect(response.trace, isNotNull);
    expect(response.trace!['primary']!.id, equals('primary'));
    expect(response.trace!['primary']!.value, orderedEquals([4, 5, 6]));

    final manual = ZenEngineResponse(
      performance: 'manual',
      result: Uint8List.fromList([9, 8, 7]),
      trace: {
        'manual': ZenEngineTrace(
          id: 'manual',
          value: Uint8List.fromList([6, 5, 4]),
        ),
      },
    );
    final roundTrip = returnZenEngineResponse(response: manual);

    expect(roundTrip.performance, equals('manual'));
    expect(roundTrip.result, orderedEquals([9, 8, 7]));
    expect(roundTrip.trace!['manual']!.id, equals('manual'));
    expect(roundTrip.trace!['manual']!.value, orderedEquals([6, 5, 4]));
  });

  test('acronym-named custom and record types round trip', () {
    final value = ApiResult(
      primary: HttpMetadata(url: 'https://primary.example', status: 200),
      fallback: HttpMetadata(url: 'https://fallback.example', status: 503),
    );

    final roundTrip = roundtripApiResult(value: value);

    expect(roundTrip.primary.url, equals('https://primary.example'));
    expect(roundTrip.primary.status, equals(200));
    expect(roundTrip.fallback!.url, equals('https://fallback.example'));
    expect(roundTrip.fallback!.status, equals(503));
  });

  // A custom type in a callback interface signature must lower to its builtin.
  // The alias is a Dart typedef, and a typedef is not a `NativeType`, so the
  // generated file does not compile if the generator emits the alias here.
  test('custom type survives a callback interface round trip', () {
    final transformer = UpperCaseCustomIdTransformer();

    final result = roundtripCustomIdThroughCallback(
      transformer: transformer,
      id: 'id-abc123',
    );

    // Rust passed the argument to Dart without a change.
    expect(transformer.received, equals(<String>['id-abc123']));
    // Dart returned a value, and Rust passed it back.
    expect(result, equals('ID-ABC123'));
  });
}
