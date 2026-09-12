import 'dart:io';

import 'package:test/test.dart';

void main() {
  test('argument ownership balances on success and failure', () async {
    // Compile a child suite with an instrumented allocator. Keep the production
    // API unchanged and restore the generated files even when a test fails.
    final originals = <File, String>{};
    try {
      for (final name in ['argument_rollback.dart', 'uniffi_runtime.dart']) {
        final file = File(name);
        final source = file.readAsStringSync();
        originals[file] = source;
        final import = RegExp(r'''import ["']package:ffi/ffi.dart["'];''');
        expect(import.allMatches(source), hasLength(1));
        var instrumented = source.replaceFirst(
          import,
          'import "package:ffi/ffi.dart" hide calloc;\n'
          'import "test/tracking_allocator.dart";',
        );
        if (name == 'uniffi_runtime.dart') {
          instrumented = instrumented.replaceFirst(
            'class UniffiHandleMap<T> {',
            'class UniffiHandleMap<T> { int get testLength => _map.length;',
          );
        } else {
          instrumented += '\nint callbackHandlesForTest() => FfiConverterCallbackInterfaceConsumer._handleMap.testLength;\n';
        }
        file.writeAsStringSync(instrumented);
      }
      final result = await Process.run(Platform.resolvedExecutable, [
        'test',
        'test/rollback_cases.dart',
        '--reporter',
        'expanded',
      ]);
      print(result.stdout);
      print(result.stderr);
      expect(result.exitCode, 0);
    } finally {
      for (final entry in originals.entries) {
        entry.key.writeAsStringSync(entry.value);
      }
    }
  }, timeout: Timeout(Duration(minutes: 3)));
}
