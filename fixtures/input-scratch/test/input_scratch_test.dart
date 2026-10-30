import 'dart:io';

import 'package:test/test.dart';

void main() {
  test('scratch allocations balance on success and injected failure', () async {
    // Compile a child suite with an instrumented allocator. Keep the production
    // API unchanged and restore the generated files even when a test fails.
    final originals = <File, String>{};
    try {
      for (final name in ['input_scratch.dart', 'uniffi_runtime.dart']) {
        final file = File(name);
        final source = file.readAsStringSync();
        originals[file] = source;
        final import = RegExp(r'''import ["']package:ffi/ffi.dart["'];''');
        expect(import.allMatches(source), hasLength(1));
        file.writeAsStringSync(
          source.replaceFirst(
            import,
            'import "package:ffi/ffi.dart" hide calloc;\n'
            'import "test/tracking_allocator.dart";',
          ),
        );
      }
      final result = await Process.run(Platform.resolvedExecutable, [
        'test',
        'test/scratch_cases.dart',
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
