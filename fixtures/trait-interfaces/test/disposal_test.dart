import 'package:test/test.dart';

import '../trait_interfaces.dart';

void main() {
  test('regular wrapper frees once and rejects later use', () {
    final before = disposalDropCount();
    final probe = DisposalProbe();
    expect(probe.greet(name: 'Dart'), 'Hello Dart');
    probe.dispose();
    expect(disposalDropCount(), before + 1);
    probe.dispose();
    expect(disposalDropCount(), before + 1);
    expect(
      () => probe.uniffiClonePointer(),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          'Cannot use a disposed DisposalProbe',
        ),
      ),
    );
    expect(() => probe.greet(name: 'Dart'), throwsStateError);
    expect(disposalDropCount(), before + 1);
  });

  test(
    'disposing one wrapper preserves an independently owned trait handle',
    () {
      final before = disposalDropCount();
      final probe = DisposalProbe();
      final greeter = probe.asGreeter();
      probe.dispose();
      probe.dispose();
      expect(disposalDropCount(), before);
      expect(greeter.greet(name: 'trait'), 'Hello trait');
      greeter.dispose();
      expect(disposalDropCount(), before + 1);
      greeter.dispose();
      expect(disposalDropCount(), before + 1);
      expect(() => greeter.greet(name: 'trait'), throwsStateError);
      expect(() => FfiConverterGreeter.lower(greeter), throwsStateError);
    },
  );
}
