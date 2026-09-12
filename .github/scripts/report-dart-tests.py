#!/usr/bin/env python3
"""Report Dart's JSON test results and optionally require complete coverage."""

import argparse
import json
import os
from pathlib import Path


def summarize(path, require_no_skips=False):
    tests = {}
    success = None
    for line in path.read_text().splitlines():
        event = json.loads(line)
        if event['type'] == 'testDone':
            # A later asynchronous error can update an already completed test.
            tests[event['testID']] = event
        elif event['type'] == 'done':
            success = event.get('success')

    visible = [event for event in tests.values() if not event.get('hidden', False)]
    skipped = sum(event.get('skipped', False) for event in visible)
    passed = sum(
        event['result'] == 'success' and not event.get('skipped', False)
        for event in visible
    )
    failed = sum(event['result'] != 'success' for event in visible)
    report = f'Dart tests: {passed} passed, {failed} failed, {skipped} skipped.'
    if success is not True or failed or passed == 0:
        return report + '\nThe test run did not complete successfully with passing tests.', 1
    if require_no_skips and skipped:
        return report + '\nThe integration run requires every test to run.', 1
    return report, 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('results', type=Path)
    parser.add_argument('--require-no-skips', action='store_true')
    args = parser.parse_args()
    report, status = summarize(args.results, args.require_no_skips)
    print(report)
    if summary := os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(summary, 'a') as output:
            output.write(report + '\n')
            if not args.require_no_skips:
                output.write(
                    '\nBDK testnet integration is opt-in. Run Test Downstream manually '
                    'with Electrum and Esplora testnet endpoints to require zero skips.\n'
                )
    return status


if __name__ == '__main__':
    raise SystemExit(main())
