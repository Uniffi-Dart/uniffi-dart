import 'package:test/test.dart';
import '../time_types.dart';

void main() {
  group('Time Types', () {
    test('basic timestamp operations', () {
      // Test returning timestamps
      final now = timeTypesNow();
      final returned = returnTimestamp(now);
      expect(returned, equals(now));
    });

    test('basic duration operations', () {
      // Test returning durations
      final duration = Duration(seconds: 10);
      final returned = returnDuration(duration);
      expect(returned, equals(duration));
    });

    test('timestamp string conversion', () {
      // Test converting timestamps to ISO 8601 strings
      final timestamp = timeTypesNow();
      final timeString = toStringTimestamp(timestamp);

      // Should be in ISO 8601 format with microseconds and Z
      expect(
        timeString,
        matches(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z'),
      );
    });

    test('pre-epoch timestamps', () {
      // Test timestamps before Unix epoch (1970-01-01)
      final preEpoch = getPreEpochTimestamp();
      final epochTime = DateTime(1970, 1, 1).millisecondsSinceEpoch;

      expect(preEpoch.millisecondsSinceEpoch, lessThan(epochTime));
    });

    test('timestamp arithmetic', () {
      // Test adding duration to timestamp
      final baseTime = timeTypesNow();
      final duration = Duration(hours: 1, minutes: 30);

      final result = add(baseTime, duration);
      expect(result.difference(baseTime), equals(duration));
    });

    test('timestamp difference calculation', () {
      // Test calculating difference between timestamps
      final time1 = timeTypesNow();
      final duration = Duration(minutes: 45);
      final time2 = add(time1, duration);

      final difference = diff(time2, time1);
      expect(difference, equals(duration));
    });

    test('timestamp equality', () {
      // Test timestamp equality comparison
      final time1 = timeTypesNow();
      final time2 = time1; // Same reference
      final time3 = add(time1, Duration(seconds: 1));

      expect(equal(time1, time2), isTrue);
      expect(equal(time1, time3), isFalse);
    });

    test('optional timestamp and duration', () {
      // Test optional timestamp and duration parameters
      final timestamp = timeTypesNow();
      final duration = Duration(seconds: 30);

      expect(optional(timestamp, duration), isTrue);
      expect(optional(null, duration), isFalse);
      expect(optional(timestamp, null), isFalse);
      expect(optional(null, null), isFalse);
    });

    test('seconds before Unix epoch operations', () {
      // Test operations with times before Unix epoch
      final preEpoch = getPreEpochTimestamp();
      final secondsBefore = getSecondsBeforeUnixEpoch(preEpoch);

      expect(secondsBefore, greaterThan(0));

      // Test setting timestamp from seconds before epoch
      final recreated = setSecondsBeforeUnixEpoch(secondsBefore);
      expect(equal(recreated, preEpoch), isTrue);
    });

    test('error handling - time overflow', () {
      // Test error handling for time overflow scenarios
      expect(() {
        // Try to add a massive duration that would cause overflow
        final maxTime = DateTime.fromMillisecondsSinceEpoch(
          DateTime.now().millisecondsSinceEpoch + 1000000000000,
        );
        final hugeDuration = Duration(days: 365 * 1000); // 1000 years
        add(maxTime, hugeDuration);
      }, throwsA(isA<ChronologicalError>()));
    });

    test('error handling - time difference error', () {
      // Test error handling for time difference calculations
      expect(() {
        final time1 = timeTypesNow();
        final time2 = add(time1, Duration(hours: -1)); // Earlier time
        diff(time2, time1); // Should fail: time2 is before time1
      }, throwsA(isA<ChronologicalError>()));
    });

    test('comprehensive timestamp workflow', () {
      // Test a complete workflow with various time operations
      final startTime = timeTypesNow();

      // Add some time
      final afterOneHour = add(startTime, Duration(hours: 1));
      final afterTwoHours = add(startTime, Duration(hours: 2));

      // Test differences
      final oneHourDiff = diff(afterOneHour, startTime);
      final twoHourDiff = diff(afterTwoHours, startTime);
      final betweenDiff = diff(afterTwoHours, afterOneHour);

      expect(oneHourDiff, equals(Duration(hours: 1)));
      expect(twoHourDiff, equals(Duration(hours: 2)));
      expect(betweenDiff, equals(Duration(hours: 1)));

      // Test string conversion
      final timeString = toStringTimestamp(afterOneHour);
      expect(timeString, isNotEmpty);
      expect(timeString, contains('T'));
      expect(timeString, endsWith('Z'));
    });

    test('duration edge cases', () {
      // Test various duration edge cases
      final zeroTime = Duration.zero;
      final returned = returnDuration(zeroTime);
      expect(returned, equals(Duration.zero));

      final microDuration = Duration(microseconds: 1);
      final returnedMicro = returnDuration(microDuration);
      expect(returnedMicro, equals(microDuration));

      final largeDuration = Duration(days: 365);
      final returnedLarge = returnDuration(largeDuration);
      expect(returnedLarge, equals(largeDuration));
    });
  });
}
