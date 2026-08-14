import json
import tempfile
import unittest
from pathlib import Path

from scripts.check_lcov_thresholds import evaluate, main, read_lcov


class CoverageThresholdTests(unittest.TestCase):
    def fixture(self, lcov: str, lines: float = 90, branches: float = 85):
        directory = tempfile.TemporaryDirectory()
        root = Path(directory.name)
        lcov_path = root / "lcov.info"
        plan_path = root / "quality-plan.json"
        lcov_path.write_text(lcov, encoding="utf-8")
        plan_path.write_text(
            json.dumps(
                {
                    "thresholds": {
                        "lineCoveragePercent": lines,
                        "branchCoveragePercent": branches,
                    }
                }
            ),
            encoding="utf-8",
        )
        return directory, lcov_path, plan_path

    def test_sums_records_and_accepts_exact_thresholds(self):
        directory, lcov_path, plan_path = self.fixture(
            "LF:50\nLH:45\nBRF:20\nBRH:17\nend_of_record\n"
            "LF:50\nLH:45\nBRF:20\nBRH:17\nend_of_record\n"
        )
        self.addCleanup(directory.cleanup)

        lines, branches = read_lcov(lcov_path)
        self.assertEqual((lines.found, lines.hit), (100, 90))
        self.assertEqual((branches.found, branches.hit), (40, 34))
        report, passed = evaluate(lcov_path, plan_path)
        self.assertTrue(passed)
        self.assertIn("lines 90/100", report)
        self.assertIn("branches 34/40", report)

    def test_rejects_either_metric_below_its_normative_threshold(self):
        directory, lcov_path, plan_path = self.fixture(
            "LF:100\nLH:89\nBRF:100\nBRH:84\nend_of_record\n"
        )
        self.addCleanup(directory.cleanup)

        _, passed = evaluate(lcov_path, plan_path)
        self.assertFalse(passed)
        self.assertEqual(main(["check", str(lcov_path), str(plan_path)]), 1)

    def test_missing_branch_evidence_is_invalid(self):
        directory, lcov_path, plan_path = self.fixture(
            "LF:100\nLH:100\nend_of_record\n"
        )
        self.addCleanup(directory.cleanup)

        with self.assertRaisesRegex(ValueError, "no instrumented items"):
            evaluate(lcov_path, plan_path)
        self.assertEqual(main(["check", str(lcov_path), str(plan_path)]), 2)


if __name__ == "__main__":
    unittest.main()
