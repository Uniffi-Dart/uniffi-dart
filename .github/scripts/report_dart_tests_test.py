import json
from pathlib import Path
import runpy
import tempfile
import unittest

summarize = runpy.run_path(str(Path(__file__).with_name('report-dart-tests.py')))['summarize']


def result(test_id, *, skipped=False, hidden=False, outcome='success'):
    return {'type': 'testDone', 'testID': test_id, 'result': outcome,
            'skipped': skipped, 'hidden': hidden}


class ReportTests(unittest.TestCase):
    def report(self, events, strict=False):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'results.jsonl'
            path.write_text(''.join(json.dumps(event) + '\n' for event in events))
            return summarize(path, strict)

    def test_passing_tests_exclude_hidden_suite_setup(self):
        report, status = self.report([
            result(0, hidden=True), result(1), result(2),
            {'type': 'done', 'success': True},
        ], strict=True)
        self.assertEqual(status, 0)
        self.assertIn('2 passed, 0 failed, 0 skipped', report)

    def test_routine_run_reports_skips(self):
        report, status = self.report([
            result(1), result(2, skipped=True), {'type': 'done', 'success': True},
        ])
        self.assertEqual(status, 0)
        self.assertIn('1 passed, 0 failed, 1 skipped', report)

    def test_integration_run_rejects_skips(self):
        _, status = self.report([
            result(1), result(2, skipped=True), {'type': 'done', 'success': True},
        ], strict=True)
        self.assertEqual(status, 1)

    def test_empty_or_only_hidden_results_cannot_pass(self):
        for events in [[], [{'type': 'done', 'success': True}],
                       [result(1, hidden=True), {'type': 'done', 'success': True}]]:
            with self.subTest(events=events):
                self.assertEqual(self.report(events)[1], 1)

    def test_truncated_results_cannot_pass(self):
        self.assertEqual(self.report([result(1)])[1], 1)

    def test_failed_test_run_cannot_pass(self):
        self.assertEqual(self.report([
            result(1), result(2, outcome='failure'), {'type': 'done', 'success': False},
        ])[1], 1)

    def test_late_error_replaces_earlier_success(self):
        report, status = self.report([
            result(1), result(1, outcome='error'), {'type': 'done', 'success': False},
        ])
        self.assertEqual(status, 1)
        self.assertIn('0 passed, 1 failed, 0 skipped', report)

    def test_failed_event_cannot_be_overridden_by_done_success(self):
        self.assertEqual(self.report([
            result(1), result(2, outcome='error'), {'type': 'done', 'success': True},
        ])[1], 1)


if __name__ == '__main__':
    unittest.main()
