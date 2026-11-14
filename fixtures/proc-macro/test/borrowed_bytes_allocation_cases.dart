import 'dart:ffi';
import 'dart:typed_data';

import 'package:test/test.dart';

import 'instrumented/proc_macro.dart';
import 'tracking_allocator.dart' as tracker;

// Run by borrowed_bytes_allocation_test.dart against the instrumented bindings;
// not a standalone suite.
void main() {
  ensureInitialized();

  // A distinctive length, so the data copy can be told apart from any other
  // allocation a generated call makes.
  const length = 37;
  final payload = Uint8List.fromList(List<int>.generate(length, (i) => i));
  const payloadSum = length * (length - 1) ~/ 2;

  late Set<int> liveBefore;
  setUp(() {
    liveBefore = tracker.calloc.live.keys.toSet();
    tracker.calloc.allocatedSizes.clear();
  });

  // The copy must have gone through the tracked arena (otherwise a balance check
  // passes vacuously), and nothing allocated during the call may still be live.
  void expectCopyFreed({required int dataBytes}) {
    final sizes = tracker.calloc.allocatedSizes;
    expect(sizes, contains(sizeOf<ForeignBytes>()),
        reason: 'ForeignBytes struct was not allocated through the arena');
    if (dataBytes > 0) {
      expect(sizes, contains(dataBytes),
          reason: 'data copy was not allocated through the arena');
    }
    final leaked = tracker.calloc.live.keys.toSet().difference(liveBefore);
    expect(leaked.map((a) => tracker.calloc.live[a]), isEmpty,
        reason: 'native blocks still live after the call (sizes shown)');
  }

  test('free function', () {
    expect(sumBorrowedBytes(data: payload), payloadSum);
    expectCopyFreed(dataBytes: length);
  });

  test('empty slice allocates only the struct', () {
    expect(sumBorrowedBytes(data: Uint8List(0)), 0);
    expectCopyFreed(dataBytes: 0);
  });

  test('void return', () {
    consumeBorrowedBytes(data: payload);
    expectCopyFreed(dataBytes: length);
  });

  test('method on an object', () {
    final obj = Object();
    expect(obj.borrowedBytesLen(data: payload), length);
    obj.dispose();
    expectCopyFreed(dataBytes: length);
  });

  test('constructor initializer-list call site', () {
    Object.fromBytes(data: payload).dispose();
    expectCopyFreed(dataBytes: length);
  });

  test('Rust error path frees the data copy', () {
    // Leading 0xFF makes the fixture return an error for a non-empty buffer.
    final failing = Uint8List.fromList([0xFF, ...payload.skip(1)]);
    expect(() => sumBorrowedBytesChecked(data: failing),
        throwsA(isA<OsExceptionBasicException>()));
    expectCopyFreed(dataBytes: length);
  });

  test('failure while lowering a later argument frees the earlier copy', () {
    // `data` is copied into the arena first; lowering `scale: -1` as a u32 then
    // throws before Rust is entered.
    expect(() => sumBorrowedBytesScaled(data: payload, scale: -1),
        throwsArgumentError);
    expectCopyFreed(dataBytes: length);

    expect(sumBorrowedBytesScaled(data: payload, scale: 2), payloadSum * 2);
    expectCopyFreed(dataBytes: length);
  });

  test('repeated calls do not accumulate', () {
    for (var i = 0; i < 1000; i++) {
      sumBorrowedBytes(data: payload);
    }
    expect(tracker.calloc.allocatedSizes.where((s) => s == length),
        hasLength(1000));
    expectCopyFreed(dataBytes: length);
  });
}
