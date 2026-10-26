import 'package:test/test.dart';

import '../callback_ffi_types.dart';
import 'callback_ffi_types_test.dart' show Values;

class Leaf implements LeafCallback {
  @override
  int value(int input) => input + 1;
}

class Nested implements NestedCallback {
  final Leaf leaf;
  Nested(this.leaf);
  @override
  LeafCallback get_() => leaf;
}

class ForeignNested implements NestedForeignCallback {
  final AbiValues leaf;
  ForeignNested(this.leaf);
  @override
  bool invoke(AbiValues value) {
    expect(identical(value, leaf), isTrue);
    return value.count(41) == 42;
  }

  @override
  AbiValues get_() => leaf;
}

void main() {
  test(
    'foreign-implementable traits cross callback argument and result slots',
    () {
      final leaf = Values();
      expect(
        checkNestedForeignCallbacks(callback: ForeignNested(leaf), leaf: leaf),
        isTrue,
      );
    },
  );
  test('callback interfaces cross callback argument and result slots', () {
    final leaf = Leaf();
    expect(checkNestedCallbacks(callback: Nested(leaf)), isTrue);
  });
}
