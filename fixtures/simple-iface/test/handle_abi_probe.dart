import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

typedef PointerCallback = Void Function(Pointer<Void>, Uint32, Pointer<Void>);
typedef FixedCallback = Void Function(Uint64, Uint32, Pointer<Void>);

void main(List<String> args) {
  final lib = DynamicLibrary.open(args.single);
  final capturedHandle = lib.lookupFunction<Uint64 Function(), int Function()>(
    'abi_captured_handle',
  );
  final capturedMarker = lib.lookupFunction<Uint32 Function(), int Function()>(
    'abi_captured_marker',
  );
  final capturedStatus = lib.lookupFunction<UintPtr Function(), int Function()>(
    'abi_captured_status',
  );
  const handle = 0x1230; // Ordinary aligned value, well below 2^32.
  const marker = 0x5678;
  final status = Pointer<Void>.fromAddress(0x2340); // Never dereferenced.
  final expected = [handle, marker, status.address];
  List<int> captured() => [
    capturedHandle(),
    capturedMarker(),
    capturedStatus(),
  ];
  final results = <Map<String, Object>>[];
  void record(
    String name,
    Object actual,
    Object expected, {
    required bool control,
  }) {
    final pass = jsonEncode(actual) == jsonEncode(expected);
    results.add({
      'name': name,
      'actual': actual,
      'expected': expected,
      'pass': pass,
      'control': control,
    });
    if (control && !pass) throw StateError('Control failed: ${results.last}');
  }

  // Regression control: the old generator mapped handles to Pointer<Void>.
  final current = lib
      .lookupFunction<
        Void Function(Pointer<Void>, Uint32, Pointer<Void>),
        void Function(Pointer<Void>, int, Pointer<Void>)
      >('abi_capture');
  final fixed = lib
      .lookupFunction<
        Void Function(Uint64, Uint32, Pointer<Void>),
        void Function(int, int, Pointer<Void>)
      >('abi_capture');
  fixed(handle, marker, status);
  record('Uint64 argument control', captured(), expected, control: true);
  current(Pointer<Void>.fromAddress(handle), marker, status);
  record(
    'Pointer handle followed by marker and status',
    captured(),
    expected,
    control: false,
  );

  final cloneCurrent = lib
      .lookupFunction<
        Pointer<Void> Function(Pointer<Void>, Pointer<Void>),
        Pointer<Void> Function(Pointer<Void>, Pointer<Void>)
      >('abi_capture_clone');
  final cloneFixed = lib
      .lookupFunction<
        Uint64 Function(Uint64, Pointer<Void>),
        int Function(int, Pointer<Void>)
      >('abi_capture_clone');
  record(
    'Uint64 clone return control',
    cloneFixed(handle, status),
    handle,
    control: true,
  );
  record('Uint64 clone arguments control', captured(), [
    handle,
    0,
    status.address,
  ], control: true);
  cloneCurrent(Pointer<Void>.fromAddress(handle), status);
  record('Pointer handle followed by status (clone/free shape)', captured(), [
    handle,
    0,
    status.address,
  ], control: false);

  final returnCurrent = lib
      .lookupFunction<Pointer<Void> Function(), Pointer<Void> Function()>(
        'abi_return_handle',
      );
  final returnFixed = lib.lookupFunction<Uint64 Function(), int Function()>(
    'abi_return_handle',
  );
  const highHandle = 0x123456789abcdef0;
  record('Uint64 return control', returnFixed(), highHandle, control: true);
  record(
    'Pointer handle return',
    returnCurrent().address,
    highHandle,
    control: false,
  );

  List<int>? callbackCapture;
  final cbCurrent = NativeCallable<PointerCallback>.isolateLocal((
    Pointer<Void> h,
    int m,
    Pointer<Void> s,
  ) {
    callbackCapture = [h.address, m, s.address];
  });
  final cbFixed = NativeCallable<FixedCallback>.isolateLocal((
    int h,
    int m,
    Pointer<Void> s,
  ) {
    callbackCapture = [h, m, s.address];
  });
  final invokeCurrent = lib
      .lookupFunction<
        Void Function(
          Pointer<NativeFunction<PointerCallback>>,
          Uint64,
          Uint32,
          Pointer<Void>,
        ),
        void Function(
          Pointer<NativeFunction<PointerCallback>>,
          int,
          int,
          Pointer<Void>,
        )
      >('abi_invoke_callback');
  final invokeFixed = lib
      .lookupFunction<
        Void Function(
          Pointer<NativeFunction<FixedCallback>>,
          Uint64,
          Uint32,
          Pointer<Void>,
        ),
        void Function(
          Pointer<NativeFunction<FixedCallback>>,
          int,
          int,
          Pointer<Void>,
        )
      >('abi_invoke_callback');
  try {
    invokeFixed(cbFixed.nativeFunction, handle, marker, status);
    record(
      'Uint64 callback control',
      callbackCapture!,
      expected,
      control: true,
    );
    invokeCurrent(cbCurrent.nativeFunction, handle, marker, status);
    record(
      'Pointer callback with low handle',
      callbackCapture!,
      expected,
      control: false,
    );
    invokeFixed(cbFixed.nativeFunction, highHandle, marker, status);
    record('Uint64 callback high handle control', callbackCapture!, [
      highHandle,
      marker,
      status.address,
    ], control: true);
    invokeCurrent(cbCurrent.nativeFunction, highHandle, marker, status);
    record('Pointer callback with high handle', callbackCapture!, [
      highHandle,
      marker,
      status.address,
    ], control: false);
  } finally {
    cbCurrent.close();
    cbFixed.close();
  }
  print(
    const JsonEncoder.withIndent('  ').convert({
      'dart': Platform.version,
      'abi': Abi.current().toString(),
      'pointerBytes': sizeOf<Pointer<Void>>(),
      'results': results,
    }),
  );
  final failures = results.where((r) => r['pass'] == false).length;
  final expectedFailures = sizeOf<Pointer<Void>>() == 4 ? 5 : 0;
  if (failures != expectedFailures)
    throw StateError('Expected $expectedFailures mismatches, got $failures');
}
