import 'package:test/test.dart';

import '../dart_async.dart';

class Factory extends AsyncObjectFactory {
  final Megaphone object;
  Factory(this.object);
  @override
  Future<Megaphone> make() async => object;
}

void main() {
  test('async foreign method returns a fixed-width object handle', () async {
    final original = await Megaphone.new_();
    final returned = await makeUsingForeignFactory(factory: Factory(original));
    try {
      expect(
        await returned.sayAfter(ms: 0, who: 'handle'),
        await original.sayAfter(ms: 0, who: 'handle'),
      );
    } finally {
      returned.dispose();
      original.dispose();
    }
  });
}
