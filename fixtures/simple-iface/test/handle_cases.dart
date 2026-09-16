import '../simple_iface.dart' as bindings;

void check(bool condition, String message) {
  if (!condition) throw StateError(message);
}

class Mapper extends bindings.ObjectMapper {
  @override
  bindings.Object map(bindings.Object value) {
    check(value.getInner() == 42, 'callback object input');
    // Lowering clones the returned handle before Rust consumes it.
    return value;
  }
}

class Delegate extends bindings.HandleDelegate {
  int calls = 0;
  @override
  int label() {
    calls++;
    return 44;
  }
}

Future<void> runHandleCases() async {
  final object = bindings.makeObject(inner: 42);
  try {
    check(object.getInner() == 42, 'object clone and method');
    print('before typed object error');
    var caught = false;
    try {
      object.fail();
    } on bindings.ProbeException catch (error) {
      check(error == bindings.ProbeException.failed, 'declared error value');
      caught = true;
    }
    check(caught, 'typed error must be delivered');
    final mapped = bindings.mapObject(mapper: Mapper(), value: object);
    check(mapped.getInner() == 42, 'callback object return');
    mapped.dispose();
  } finally {
    object.dispose();
  }

  final label = bindings.makeHandleLabel();
  check(label.label() == 42, 'Rust-only trait handle');
  label.dispose();
  final nativeDelegate = bindings.makeHandleDelegate();
  check(nativeDelegate.label() == 43, 'Rust-backed foreign trait handle');
  check(
    bindings.callHandleDelegate(value: nativeDelegate) == 43,
    'Rust-backed foreign trait lowering',
  );
  // The abstract foreign trait does not expose disposal; its native wrapper does.
  (nativeDelegate as dynamic).dispose();
  final delegate = Delegate();
  check(
    bindings.callHandleDelegate(value: delegate) == 44,
    'Dart trait callback',
  );
  check(delegate.calls == 1, 'original Dart trait state');
  check(
    identical(bindings.echoHandleDelegate(value: delegate), delegate),
    'foreign trait round trip retains identity',
  );
  final futureObject = await bindings.makeObjectAsync(inner: 57);
  check(futureObject.getInner() == 57, 'future complete returns object handle');
  futureObject.dispose();
  print('PASS: handles, typed error, object callback, traits, pending future');
}

Future<void> main() => runHandleCases();
