import 'package:test/test.dart';

import '../proc_macro.dart';

class DartStatusTrait implements ForeignStatusTrait {
  int value = 0;

  @override
  String describeStatus(int status) {
    if (status != 0) throw InvalidInputBasicException();
    return 'status: $status';
  }

  @override
  int addStatus(int status) => value + status;

  @override
  void setStatus(int status) {
    value = status;
  }
}

void main() {
  ensureInitialized();

  test('free functions preserve status and normalized suffix arguments', () {
    expect(combineStatus(status: 1, status1: 2, status2: 3), 123);
    checkStatus(status: 0);
    expect(
      () => checkStatus(status: 7),
      throwsA(isA<InvalidInputBasicException>()),
    );
  });

  test('constructors and object methods preserve status arguments', () {
    final object = StatusObject(status: 10);
    final named = StatusObject.withStatus(status: 20, status1: 3);
    addTearDown(object.dispose);
    addTearDown(named.dispose);

    expect(object.addStatus(status: 2), 12);
    expect(named.addStatus(status: 4), 27);
    object.setStatus(status: 30);
    expect(object.addStatus(status: 5), 35);
  });

  test('Rust trait methods preserve status arguments', () {
    final object = StatusObject(status: 40);
    final trait = object.asStatusTrait();
    addTearDown(object.dispose);
    addTearDown(trait.dispose);

    expect(trait.addStatus(status: 2), 42);
    trait.setStatus(status: 50);
    expect(object.addStatus(status: 3), 53);
  });

  test('Dart callbacks preserve status arguments', () {
    final callback = DartStatusTrait();
    expect(callForeignStatus(callback: callback, status: 80), 82);
    expect(callback.value, 80);
    expect(describeForeignStatus(callback: callback, status: 0), 'status: 0');
    expect(
      () => describeForeignStatus(callback: callback, status: 7),
      throwsA(isA<InvalidInputBasicException>()),
    );
  });

  test('Rust implementations of foreign traits preserve status arguments', () {
    final object = StatusObject(status: 60);
    final trait = object.asForeignStatusTrait();
    addTearDown(object.dispose);

    expect(trait.describeStatus(0), 'status: 0');
    expect(
      () => trait.describeStatus(7),
      throwsA(isA<InvalidInputBasicException>()),
    );
    expect(trait.addStatus(2), 62);
    trait.setStatus(70);
    expect(object.addStatus(status: 3), 73);
  });
}
