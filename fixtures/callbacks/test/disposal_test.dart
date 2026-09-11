import 'package:test/test.dart';

import '../callbacks.dart';

void main() {
  test(
    'Rust-backed foreign trait rejects calls and lowering after disposal',
    () {
      final owner = InMemoryEventPersister();
      final persister = owner.asPersister();
      final other = owner.asPersister();
      owner.dispose();
      persister.save('before disposal');
      // The abstract foreign interface also supports Dart implementations, so
      // disposal is available only on this Rust-backed implementation.
      (persister as dynamic).dispose();
      (persister as dynamic).dispose();
      expect(() => persister.save('after disposal'), throwsStateError);
      expect(() => persister.load(), throwsStateError);
      expect(
        () =>
            saveAndLoadPersister(persister: persister, event: 'after disposal'),
        throwsStateError,
      );
      expect(other.load(), ['before disposal']);
      (other as dynamic).dispose();
    },
  );
}
