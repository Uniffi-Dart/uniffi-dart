import 'dart:io';

import 'package:test/test.dart';

void main() {
  test('disposal survives injected status allocation failure', () async {
    // Run a child suite in an isolated copy of this package with an
    // instrumented allocator. The child's build hooks replace the native
    // library, so it must not share this run's package directory.
    final work = Directory.systemTemp.createTempSync('disposal_allocation');
    try {
      await _copyPackage(Directory.current, work);
      final pubGet = await Process.run(Platform.resolvedExecutable, [
        'pub',
        'get',
        '--offline',
      ], workingDirectory: work.path);
      print(pubGet.stdout);
      print(pubGet.stderr);
      expect(pubGet.exitCode, 0);
      for (final name in ['trait_interfaces.dart', 'uniffi_runtime.dart']) {
        final file = File('${work.path}/$name');
        final source = file.readAsStringSync();
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
        'test/disposal_allocation_cases.dart',
        '--reporter',
        'expanded',
      ], workingDirectory: work.path);
      print(result.stdout);
      print(result.stderr);
      expect(result.exitCode, 0);
    } finally {
      work.deleteSync(recursive: true);
    }
  }, timeout: Timeout(Duration(minutes: 3)));
}

Future<void> _copyPackage(Directory from, Directory to) async {
  to.createSync(recursive: true);
  for (final entity in from.listSync()) {
    final name = entity.uri.pathSegments
        .where((segment) => segment.isNotEmpty)
        .last;
    if (name == '.dart_tool') {
      continue;
    }
    if (entity is Directory) {
      await _copyPackage(entity, Directory('${to.path}/$name'));
    } else {
      File(entity.path).copySync('${to.path}/$name');
    }
  }
}
