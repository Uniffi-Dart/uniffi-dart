import 'dart:io';

import 'package:test/test.dart';

// Runs borrowed_bytes_allocation_cases.dart in a child `dart test` against a copy
// of the generated bindings whose native allocations go through
// tracking_allocator.dart, as the argument-rollback fixture does. The copy lives in
// its own directory rather than rewriting the generated files in place, because
// proc_macro_test.dart loads those files concurrently in the same run.
void main() {
  test('borrowed &[u8] native copies are freed on every call shape', () async {
    final instrumented = Directory('test/instrumented');
    try {
      instrumented.createSync(recursive: true);
      for (final name in ['proc_macro.dart', 'uniffi_runtime.dart']) {
        final source = File(name).readAsStringSync();
        final import = RegExp(r'''import ["']package:ffi/ffi.dart["'];''');
        expect(import.allMatches(source), hasLength(1), reason: name);
        File('${instrumented.path}/$name').writeAsStringSync(
          source.replaceFirst(
            import,
            'import "package:ffi/ffi.dart" hide calloc, using;\n'
            'import "../tracking_allocator.dart";',
          ),
        );
      }
      final result = await Process.run(Platform.resolvedExecutable, [
        'test',
        'test/borrowed_bytes_allocation_cases.dart',
        '--reporter',
        'expanded',
      ]);
      print(result.stdout);
      print(result.stderr);
      expect(result.exitCode, 0);
    } finally {
      if (instrumented.existsSync()) instrumented.deleteSync(recursive: true);
    }
  }, timeout: Timeout(Duration(minutes: 3)));
}
