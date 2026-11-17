import 'package:test/test.dart';
import '../map_type.dart';

void main() {
  test('map roundtrip preserves entries', () {
    final m = {'a': 1, 'b': 2};
    final out = roundtripMap(m: m);
    expect(out['a'], 1);
    expect(out['b'], 2);
    expect(out.length, 2);
  });

  test('map count', () {
    expect(countEntries(m: {'x': 10, 'y': 20, 'z': 30}), 3);
  });

  test('map with record values', () {
    final out = mapWithRecordValues(m: {'origin': Point(x: 0, y: 0), 'unit': Point(x: 1, y: 1)});
    expect(out['unit']!.x, 1);
    expect(out['unit']!.y, 1);
    expect(out.length, 2);
  });

  test('nested map round-trips (variable-length values)', () {
    final out = roundtripNestedMap(m: {
      'a': {'x': 1, 'y': 2},
      'b': {'z': 3},
    });
    expect(out['a']!['x'], 1);
    expect(out['a']!['y'], 2);
    expect(out['b']!['z'], 3);
    expect(out.length, 2);
  });

  test('map with optional values round-trips (null preserved, distinct from absent)', () {
    final out = roundtripOptionalMap(m: {'present': 7, 'absent': null});
    expect(out['present'], 7);
    expect(out['absent'], null);
    expect(out.containsKey('absent'), true);
    expect(out.length, 2);
  });

  test('empty map round-trips at the length-0 boundary', () {
    expect(roundtripMap(m: {}), isEmpty);
    expect(countEntries(m: {}), 0);
  });
}
