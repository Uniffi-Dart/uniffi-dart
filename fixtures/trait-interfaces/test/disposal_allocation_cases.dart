import 'package:test/test.dart';

import '../trait_interfaces.dart';
import 'tracking_allocator.dart' as tracker;

void main() {
  tearDown(tracker.calloc.reset);

  test('object disposal stays retryable when status allocation fails', () {
    final before = disposalDropCount();
    final probe = DisposalProbe();
    tracker.calloc.failAfter = 0;
    expect(() => probe.dispose(), throwsStateError);
    tracker.calloc.failAfter = null;
    probe.dispose();
    expect(disposalDropCount(), before + 1);
    probe.dispose();
    expect(disposalDropCount(), before + 1);
    expect(tracker.calloc.live, isEmpty);
  });

  test('trait disposal stays retryable when status allocation fails', () {
    final before = disposalDropCount();
    final probe = DisposalProbe();
    final greeter = probe.asGreeter();
    tracker.calloc.failAfter = 0;
    expect(() => greeter.dispose(), throwsStateError);
    tracker.calloc.failAfter = null;
    greeter.dispose();
    greeter.dispose();
    // The probe still holds its own reference.
    expect(disposalDropCount(), before);
    probe.dispose();
    expect(disposalDropCount(), before + 1);
    expect(tracker.calloc.live, isEmpty);
  });
}
