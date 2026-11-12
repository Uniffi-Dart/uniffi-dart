import 'package:test/test.dart';

import 'u64_cases.dart';

void main() {
  for (final entry in u64Cases.entries) {
    test(entry.key, entry.value);
  }
}
