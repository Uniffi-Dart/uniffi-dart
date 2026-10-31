import 'package:test/test.dart';

import '../callbacks.dart';
import 'tracking_allocator.dart' as tracker;

void main() {
  tearDown(tracker.calloc.reset);

  test(
    'foreign trait disposal stays retryable when status allocation fails',
    () {
      final before = persisterDropCount();
      final owner = InMemoryEventPersister();
      final persister = owner.asPersister();
      tracker.calloc.failAfter = 0;
      expect(() => (persister as dynamic).dispose(), throwsStateError);
      tracker.calloc.failAfter = null;
      (persister as dynamic).dispose();
      expect(() => persister.load(), throwsStateError);
      (persister as dynamic).dispose();
      // The owner still holds its own reference.
      expect(persisterDropCount(), before);
      owner.dispose();
      expect(persisterDropCount(), before + 1);
      expect(tracker.calloc.live, isEmpty);
    },
  );
}
