import 'package:test/test.dart';
import '../dart_async.dart';

Future<Duration> measureTime(Future<void> Function() action) async {
  final start = DateTime.now();
  await action();
  final end = DateTime.now();
  return end.difference(start);
}

Future<void> main() async {
  await ensureInitialized();

  test('greet', () async {
    final result = greet(who: "Somebody");
    expect(result, "Hello, Somebody");
  });

  test('always_ready', () async {
    final time = await measureTime(() async {
      final result = await alwaysReady();
      expect(result, true);
    });

    expect(time.inMilliseconds < 200, true);
  });

  test('void', () async {
    final time = await measureTime(() async {
      await void_();
    });
    // Less than or equal to time
    expect(time.inMilliseconds <= 10, true);
  });

  test('sleep', () async {
    final time = await measureTime(() async {
      await sleep(ms: 200);
    });

    expect(time.inMilliseconds > 200 && time.inMilliseconds < 300, true);
  });

  test('sequential_future', () async {
    final time = await measureTime(() async {
      final resultAlice = await sayAfter(ms: 100, who: 'Alice');
      final resultBob = await sayAfter(ms: 200, who: 'Bob');
      expect(resultAlice, 'Hello, Alice!');
      expect(resultBob, 'Hello, Bob!');
    });
    expect(time.inMilliseconds > 300 && time.inMilliseconds < 400, true);
  });

  test('concurrent_future', () async {
    final time = await measureTime(() async {
      final results = await Future.wait([
        sayAfter(ms: 100, who: 'Alice'),
        sayAfter(ms: 200, who: 'Bob'),
      ]);

      expect(results[0], 'Hello, Alice!');
      expect(results[1], 'Hello, Bob!');
    });

    expect(time.inMilliseconds >= 200 && time.inMilliseconds <= 300, true);
  });

  test('with_tokio_runtime', () async {
    final time = await measureTime(() async {
      final resultAlice = await sayAfterWithTokio(ms: 200, who: 'Alice');
      expect(resultAlice, 'Hello, Alice (with Tokio)!');
    });
    expect(time.inMilliseconds > 200 && time.inMilliseconds < 300, true);
  });

  test('fallible_function_and_method', () async {
    final time1 = await measureTime(() async {
      try {
        await fallibleMe(doFail: false);
        expect(true, true);
      } catch (exception) {
        expect(false, true); // should never be reached
      }
    });
    expect(time1.inMilliseconds <= 100, true);

    final time2 = await measureTime(() async {
      try {
        await fallibleMe(doFail: true);
        expect(false, true); // should never be reached
      } catch (exception) {
        expect(true, true);
      }
    });
    expect(time2.inMilliseconds <= 100, true);
  });

  test('record', () async {
    final time = await measureTime(() async {
      final result = await newMyRecord(a: 'foo', b: 42);
      expect(result.a, 'foo');
      expect(result.b, 42);
    });
    // Heads-up: Sometimes this test will fail if for whatever reason, something on the host system pauses the execution of the async funtions.
    print('record: ${time.inMilliseconds}ms');
    expect(time.inMilliseconds <= 100, true);
  });

  test('broken_sleep', () async {
    final time = await measureTime(() async {
      await brokenSleep(ms: 100, failAfter: 0); // calls the waker twice immediately
      await sleep(ms: 100); // wait for possible failure

      await brokenSleep(ms: 100, failAfter: 100); // calls the waker a second time after 1s
      await sleep(ms: 200); // wait for possible failure
    });
    expect(time.inMilliseconds >= 400 && time.inMilliseconds <= 600, true);
  });

  test('udl_async_function', () async {
    final time = await measureTime(() async {
      final result = await alwaysReady();
      expect(result, true);
    });
    expect(time.inMilliseconds < 100, true);
  });

  test('proc_macro_megaphone_async_constructor', () async {
    final time = await measureTime(() async {
      final megaphone = await Megaphone.new_();
      expect(megaphone, isNotNull);
    });
    expect(time.inMilliseconds < 100, true);
  });

  test('proc_macro_megaphone_secondary_constructor', () async {
    final time = await measureTime(() async {
      final megaphone = await Megaphone.secondary();
      expect(megaphone, isNotNull);
    });
    expect(time.inMilliseconds < 100, true);
  });

  test('proc_macro_megaphone_async_methods', () async {
    final megaphone = await Megaphone.new_();

    // Test async method with timing
    final time = await measureTime(() async {
      final result = await megaphone.sayAfter(ms: 100, who: 'Alice');
      expect(result, 'HELLO, ALICE!');
    });
    expect(time.inMilliseconds >= 100 && time.inMilliseconds < 200, true);

    // Test async silence method
    final silenceTime = await measureTime(() async {
      final result = await megaphone.silence();
      expect(result, '');
    });
    expect(silenceTime.inMilliseconds < 50, true);
  });

  test('proc_macro_megaphone_sync_method', () async {
    final megaphone = await Megaphone.new_();

    // Test sync method (should be immediate)
    final time = await measureTime(() async {
      final result = megaphone.sayNow(who: 'Bob');
      expect(result, 'HELLO, BOB!');
    });
    expect(time.inMilliseconds < 50, true);
  });

  test('proc_macro_megaphone_tokio_method', () async {
    final megaphone = await Megaphone.new_();

    final time = await measureTime(() async {
      final result = await megaphone.sayAfterWithTokio(ms: 100, who: 'Charlie');
      expect(result, 'HELLO, CHARLIE (WITH TOKIO)!');
    });
    expect(time.inMilliseconds >= 100 && time.inMilliseconds < 200, true);
  });

  test('proc_macro_megaphone_fallible_method', () async {
    final megaphone = await Megaphone.new_();

    // Test success case
    final result = await megaphone.fallibleMe(doFail: false);
    expect(result, 42);

    // Test failure case
    try {
      await megaphone.fallibleMe(doFail: true);
      expect(false, true); // Should never reach here
    } catch (e) {
      expect(true, true); // Expected to throw
    }
  });

  test('udl_megaphone_async_constructors', () async {
    // Test primary constructor
    final time1 = await measureTime(() async {
      final udlMegaphone = await UdlMegaphone.new_();
      expect(udlMegaphone, isNotNull);
    });
    expect(time1.inMilliseconds < 100, true);

    // Test secondary constructor
    final time2 = await measureTime(() async {
      final udlMegaphone = await UdlMegaphone.secondary();
      expect(udlMegaphone, isNotNull);
    });
    expect(time2.inMilliseconds < 100, true);
  });

  test('udl_megaphone_async_method', () async {
    final udlMegaphone = await UdlMegaphone.new_();

    final time = await measureTime(() async {
      final result = await udlMegaphone.sayAfter(ms: 100, who: 'Dave');
      expect(result, 'HELLO, DAVE (FROM UDL MEGAPHONE)!');
    });
    expect(time.inMilliseconds >= 100 && time.inMilliseconds < 200, true);
  });

  test('async_object_creation_functions', () async {
    // Test sync object creation
    final syncMegaphone = newMegaphone();
    expect(syncMegaphone, isNotNull);

    // Test async object creation
    final asyncMegaphone = await asyncNewMegaphone();
    expect(asyncMegaphone, isNotNull);

    // Test conditional async object creation
    final maybeMegaphone1 = await asyncMaybeNewMegaphone(y: true);
    expect(maybeMegaphone1, isNotNull);

    final maybeMegaphone2 = await asyncMaybeNewMegaphone(y: false);
    expect(maybeMegaphone2, isNull);
  });

  test('async_function_with_object_parameter', () async {
    final megaphone = await Megaphone.new_();

    final time = await measureTime(() async {
      final result = await sayAfterWithMegaphone(megaphone: megaphone, ms: 100, who: 'Eve');
      expect(result, 'HELLO, EVE!');
    });
    expect(time.inMilliseconds >= 100 && time.inMilliseconds < 200, true);
  });

  test('fallible_struct_creation', () async {
    // Test success case
    final successResult = await fallibleStruct(doFail: false);
    expect(successResult, isNotNull);

    // Test failure case
    try {
      await fallibleStruct(doFail: true);
      expect(false, true); // Should never reach here
    } catch (e) {
      expect(true, true); // Expected to throw
    }
  });

  test('fallible_async_constructor', () async {
    // This constructor always fails
    try {
      await FallibleMegaphone.new_();
      expect(false, true); // Should never reach here
    } catch (e) {
      expect(true, true); // Expected to throw
    }
  });

  test('decode sequence of records containing payload enums', () {
    final items = listAsyncItems();
    expect(items.length, 2);

    final firstState = items[0].state as ReadyAsyncItemState;
    expect(items[0].id, 1);
    expect(firstState.timestampMs, 1111);

    final secondState = items[1].state as PendingAsyncItemState;
    expect(items[1].id, 2);
    expect(secondState.reason, 'syncing');
  });
}
