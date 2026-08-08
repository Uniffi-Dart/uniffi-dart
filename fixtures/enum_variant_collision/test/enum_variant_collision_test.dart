import 'package:test/test.dart';
import '../enum_variant_collision.dart';

void main() {
  test('record collision, looping past a second-order clash, round-trips', () {
    // `Field` -> base `FieldCondition` (record) -> `FieldConditionVariant`
    // (also a record) -> `FieldConditionVariant2`. Round-trips through both the
    // variant class's lower/write and the FfiConverter's `read` dispatch.
    final c = FieldConditionVariant2(FieldCondition(key: "k"));
    final out = roundtripCondition(c: c);
    expect(out, isA<FieldConditionVariant2>());
    expect((out as FieldConditionVariant2).condition.key, "k");
  });

  test('collision with an enum type is disambiguated (enum branch of reserved set)', () {
    // `Kind` -> base `KindCondition` (a flat enum) -> `KindConditionVariant`.
    final c = KindConditionVariant(KindCondition.a);
    final out = roundtripCondition(c: c);
    expect(out, isA<KindConditionVariant>());
    expect((out as KindConditionVariant).kind, KindCondition.a);
  });

  test('non-colliding variant is unaffected', () {
    final out = roundtripCondition(c: PlainCondition(42));
    expect((out as PlainCondition).value, 42);
  });
}
