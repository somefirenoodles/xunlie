"""Unit tests for fail-closed orchestrated review validation."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import unittest

from scripts import validate_quality_system as validator


class ReviewQuorumTests(unittest.TestCase):
    def setUp(self) -> None:
        validator.ERRORS.clear()
        with (validator.ROOT / "quality/roles.json").open(encoding="utf-8") as handle:
            self.roles = json.load(handle)
        self.candidate_sha = subprocess.check_output(
            ["git", "-C", str(validator.ROOT), "rev-parse", "HEAD"], text=True
        ).strip()
        evidence = "README.md"
        digest = "sha256:" + hashlib.sha256((validator.ROOT / evidence).read_bytes()).hexdigest()
        reviewer = {
            "reviewerId": "/review/security",
            "task": "adversarial security review",
            "scope": "domain integrity",
            "candidate": "somefirenoodles/xunlie@" + self.candidate_sha,
            "verdict": "GO",
            "commands": ["cargo test --workspace --locked"],
            "findings": [],
            "timestamp": "2026-08-14T12:00:00Z",
            "evidence": evidence,
            "evidenceSha256": digest,
        }
        second = copy.deepcopy(reviewer)
        second["reviewerId"] = "/review/reproducibility"
        second["task"] = "reproducibility review"
        self.record = {
            "recordId": "TEST-G3",
            "gateId": "G3",
            "decision": "MERGE",
            "candidate": reviewer["candidate"],
            "author": "/author",
            "criticalChange": True,
            "approvalMode": "orchestrated-agent-review/v1",
            "approver": "orchestrated-review-quorum",
            "reviewers": [reviewer, second],
            "createdAt": "2026-08-14T12:01:00Z",
        }

    def tearDown(self) -> None:
        validator.ERRORS.clear()

    def test_accepts_independent_unanimous_quorum(self) -> None:
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertEqual([], validator.ERRORS)

    def test_rejects_author_as_reviewer(self) -> None:
        self.record["reviewers"][0]["reviewerId"] = self.record["author"]
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("author cannot review" in item for item in validator.ERRORS))

    def test_rejects_non_unanimous_or_open_high_finding(self) -> None:
        self.record["reviewers"][0]["verdict"] = "REWORK"
        self.record["reviewers"][1]["findings"] = [
            {"id": "P1", "severity": "high", "status": "open"}
        ]
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("unanimous GO" in item for item in validator.ERRORS))
        self.assertTrue(any("open blocking finding P1" in item for item in validator.ERRORS))

    def test_rejects_tampered_evidence(self) -> None:
        self.record["reviewers"][0]["evidenceSha256"] = "sha256:" + "0" * 64
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("evidence digest mismatch" in item for item in validator.ERRORS))

    def test_rejects_incomplete_critical_quorum(self) -> None:
        self.record["criticalChange"] = False
        self.record["reviewers"].pop()
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("below required 2" in item for item in validator.ERRORS))

    def test_accepts_complete_gate_metadata(self) -> None:
        self.record.update(
            {
                "pullRequest": "https://github.com/somefirenoodles/xunlie/pull/11",
                "requirements": ["REQ-F-004"],
                "risks": ["RISK-001"],
                "residualRisks": [
                    {"id": "RISK-001", "disposition": "bounded", "rationale": "Scoped to M2."}
                ],
                "tools": [{"id": "TOOL-RUST", "version": "1.97.1"}],
                "externalEvidence": [
                    {
                        "name": "required checks",
                        "url": "https://github.com/somefirenoodles/xunlie/actions/runs/1",
                        "integrity": "github-actions-head:" + self.candidate_sha,
                    }
                ],
            }
        )
        validator.validate_gate_metadata(
            self.record,
            "TEST-G3",
            {"REQ-F-004"},
            {"RISK-001"},
            {"TOOL-RUST"},
        )
        self.assertEqual([], validator.ERRORS)

    def test_rejects_nonexistent_candidate_commit(self) -> None:
        candidate = "somefirenoodles/xunlie@" + "f" * 40
        self.record["candidate"] = candidate
        for reviewer in self.record["reviewers"]:
            reviewer["candidate"] = candidate
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("candidate commit does not exist" in item for item in validator.ERRORS))

    def test_rejects_non_evidence_change_after_candidate(self) -> None:
        parent_sha = subprocess.check_output(
            ["git", "-C", str(validator.ROOT), "rev-parse", "HEAD^"], text=True
        ).strip()
        candidate = "somefirenoodles/xunlie@" + parent_sha
        self.record["candidate"] = candidate
        for reviewer in self.record["reviewers"]:
            reviewer["candidate"] = candidate
        validator.validate_review_quorum(self.record, "TEST-G3", self.roles)
        self.assertTrue(any("invalidates reviewer verdicts" in item for item in validator.ERRORS))

    def test_rejects_actions_evidence_from_another_candidate(self) -> None:
        self.record.update(
            {
                "pullRequest": "https://github.com/somefirenoodles/xunlie/pull/11",
                "requirements": ["REQ-F-004"],
                "risks": ["RISK-001"],
                "residualRisks": [
                    {"id": "RISK-001", "disposition": "bounded", "rationale": "Scoped to M2."}
                ],
                "tools": [{"id": "TOOL-RUST", "version": "1.97.1"}],
                "externalEvidence": [
                    {
                        "name": "wrong run",
                        "url": "https://github.com/somefirenoodles/xunlie/actions/runs/1",
                        "integrity": "github-actions-head:" + "b" * 40,
                    }
                ],
            }
        )
        validator.validate_gate_metadata(
            self.record,
            "TEST-G3",
            {"REQ-F-004"},
            {"RISK-001"},
            {"TOOL-RUST"},
        )
        self.assertTrue(any("different candidate SHA" in item for item in validator.ERRORS))

    def test_rejects_undecided_risk_and_unpinned_tool(self) -> None:
        self.record.update(
            {
                "pullRequest": "https://github.com/somefirenoodles/xunlie/pull/11",
                "requirements": ["REQ-F-004"],
                "risks": ["RISK-001"],
                "residualRisks": [],
                "tools": [{"id": "TOOL-RUST", "version": ""}],
                "externalEvidence": [],
            }
        )
        validator.validate_gate_metadata(
            self.record,
            "TEST-G3",
            {"REQ-F-004"},
            {"RISK-001"},
            {"TOOL-RUST"},
        )
        self.assertTrue(any("residualRisks must be a non-empty list" in item for item in validator.ERRORS))
        self.assertTrue(any("exact version is required" in item for item in validator.ERRORS))
        self.assertTrue(any("externalEvidence must be a non-empty list" in item for item in validator.ERRORS))


if __name__ == "__main__":
    unittest.main()
