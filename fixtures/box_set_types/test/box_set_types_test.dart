import 'package:test/test.dart';

import '../box_set_types.dart';

void main() {
  group('Box<T>', () {
    test('boxed record and enum fields round-trip as their inner types', () {
      final out = roundtripBoxes(
        boxes: Boxes(
          shape: CircleShape(1.5),
          origin: Point(x: 1, y: 2),
          target: Point(x: 3, y: 4),
        ),
      );
      expect((out.shape as CircleShape).radius, 1.5);
      expect(out.origin.x, 1);
      expect(out.origin.y, 2);
      expect(out.target?.x, 3);
      expect(out.target?.y, 4);

      final none = roundtripBoxes(
        boxes: Boxes(shape: SquareShape(2.0), origin: Point(x: 0, y: 0)),
      );
      expect((none.shape as SquareShape).side, 2.0);
      expect(none.target, isNull);
    });

    test('a recursive record through an optional boxed field', () {
      final tree = TreeNode(
        label: "root",
        child: TreeNode(
          label: "mid",
          child: TreeNode(label: "leaf"),
        ),
      );
      final out = roundtripTree(tree: tree);
      expect(out.label, "root");
      expect(out.child?.label, "mid");
      expect(out.child?.child?.label, "leaf");
      expect(out.child?.child?.child, isNull);
      expect(treeDepth(tree: tree), 3);
    });

    test('a boxed return value', () {
      final p = boxedPoint(x: 5, y: 6);
      expect(p.x, 5);
      expect(p.y, 6);
    });
  });

  group('Set<T>', () {
    test('sets of a primitive and of a custom string type in a record', () {
      final out = roundtripSets(
        sets: Sets(ids: {1, 2, 3}, tags: {"b", "a"}, maybeIds: {7}),
      );
      expect(out.ids, isA<Set<int>>());
      expect(out.ids, {1, 2, 3});
      expect(out.tags, {"a", "b"});
      expect(out.maybeIds, {7});

      final empty = roundtripSets(
        sets: Sets(ids: {}, tags: {}),
      );
      expect(empty.ids, isEmpty);
      expect(empty.tags, isEmpty);
      expect(empty.maybeIds, isNull);
    });

    test('set arguments and returns', () {
      expect(roundtripIdSet(ids: {4, 5}), {4, 5});
      expect(roundtripTagSet(tags: {"x", "y"}), {"x", "y"});
      expect(sumIds(ids: {1, 2, 3}), 6);
    });

    test('a set built in Rust drops duplicates', () {
      final tags = uniqueTags(tags: ["a", "b", "a"]);
      expect(tags, isA<Set<String>>());
      expect(tags, {"a", "b"});
    });
  });
}
