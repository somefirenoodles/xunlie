#!/usr/bin/env python3
"""Validate Xunlie's quality governance contracts using only Python stdlib."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
QUALITY = ROOT / "quality"
ERRORS: list[str] = []


def error(message: str) -> None:
    ERRORS.append(message)


def load(relative: str) -> dict[str, Any]:
    path = ROOT / relative
    try:
        with path.open("r", encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, json.JSONDecodeError) as exc:
        error(f"{relative}: cannot load JSON: {exc}")
        return {}
    if not isinstance(value, dict):
        error(f"{relative}: root must be an object")
        return {}
    return value


def index(items: Iterable[dict[str, Any]], key: str, label: str, pattern: str | None = None) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for position, item in enumerate(items):
        value = item.get(key)
        if not isinstance(value, str) or not value:
            error(f"{label}[{position}]: missing string {key}")
            continue
        if pattern and not re.fullmatch(pattern, value):
            error(f"{label}: invalid id {value!r}")
        if value in result:
            error(f"{label}: duplicate id {value}")
        result[value] = item
    return result


def require_refs(values: Any, valid: set[str], context: str) -> None:
    if not isinstance(values, list) or not values:
        error(f"{context}: must be a non-empty list")
        return
    if len(values) != len(set(values)):
        error(f"{context}: duplicate references")
    for value in values:
        if value not in valid:
            error(f"{context}: unknown reference {value!r}")


def check_acyclic(components: dict[str, dict[str, Any]]) -> None:
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(component_id: str, path: list[str]) -> None:
        if component_id in visiting:
            error("architecture: dependency cycle " + " -> ".join(path + [component_id]))
            return
        if component_id in visited:
            return
        visiting.add(component_id)
        for dependency in components[component_id].get("allowedDependencies", []):
            if dependency not in components:
                error(f"architecture: {component_id} references unknown dependency {dependency}")
            else:
                visit(dependency, path + [component_id])
        visiting.remove(component_id)
        visited.add(component_id)

    for component_id in components:
        visit(component_id, [])


def safe_evidence_path(relative: str, record: str) -> None:
    candidate = (ROOT / relative).resolve()
    try:
        candidate.relative_to(ROOT)
    except ValueError:
        error(f"{record}: evidence path escapes repository: {relative}")
        return
    if not candidate.exists():
        error(f"{record}: evidence path does not exist: {relative}")


def main() -> int:
    plan = load("quality/quality-plan.json")
    requirements_doc = load("quality/requirements.json")
    architecture = load("quality/architecture-rules.json")
    stages_doc = load("quality/stages.json")
    trace = load("quality/traceability.json")
    risks_doc = load("quality/risks.json")
    roles_doc = load("quality/roles.json")
    decisions_doc = load("quality/decisions.json")
    tools_doc = load("quality/tool-registry.json")

    plan_version = plan.get("planVersion")
    for label, value in {
        "stages.planVersion": stages_doc.get("planVersion"),
        "requirements.baselineVersion": requirements_doc.get("baselineVersion"),
        "architecture.baselineVersion": architecture.get("baselineVersion"),
        "traceability.baselineVersion": trace.get("baselineVersion"),
        "risks.baselineVersion": risks_doc.get("baselineVersion"),
        "roles.baselineVersion": roles_doc.get("baselineVersion"),
        "decisions.baselineVersion": decisions_doc.get("baselineVersion"),
        "tools.baselineVersion": tools_doc.get("baselineVersion"),
    }.items():
        if value != plan_version:
            error(f"{label}={value!r} differs from planVersion={plan_version!r}")

    if plan.get("nonCompensatory") is not True:
        error("quality plan must be non-compensatory")
    perfect = plan.get("perfectScore", {})
    if perfect.get("score") != 100 or perfect.get("requiresAllMandatoryPass") is not True:
        error("perfect score must require 100 and all mandatory controls")

    controls = index(plan.get("controls", []), "id", "controls", r"CTRL-[A-Z]+-[0-9]{3}")
    requirements = index(requirements_doc.get("requirements", []), "id", "requirements", r"REQ-[FQ]-[0-9]{3}")
    components = index(architecture.get("components", []), "id", "components", r"COMP-[A-Z]+")
    invariants = index(architecture.get("invariants", []), "id", "invariants", r"INV-ARCH-[0-9]{3}")
    stages = index(stages_doc.get("stages", []), "id", "stages", r"G[0-7]")
    tests = index(trace.get("tests", []), "id", "tests", r"TEST-[A-Z0-9-]+")
    risks = index(risks_doc.get("risks", []), "id", "risks", r"RISK-[0-9]{3}")
    decisions = index(decisions_doc.get("decisions", []), "id", "decisions", r"DEC-[0-9]{3}")
    tools = index(tools_doc.get("tools", []), "id", "tools", r"TOOL-[A-Z0-9-]+")

    if set(stages) != {f"G{i}" for i in range(8)}:
        error("stages must define exactly G0 through G7")

    for requirement_id, requirement in requirements.items():
        if requirement.get("type") not in {"functional", "quality"}:
            error(f"{requirement_id}: invalid type")
        acceptance = requirement.get("acceptance")
        if not isinstance(acceptance, list) or len(acceptance) < 2 or not all(isinstance(x, str) and x.strip() for x in acceptance):
            error(f"{requirement_id}: needs at least two non-empty acceptance criteria")
        if not requirement.get("ownerRole") or requirement.get("status") not in {"proposed", "approved", "implemented", "verified", "retired"}:
            error(f"{requirement_id}: ownerRole/status missing or invalid")

    check_acyclic(components)

    for stage_id, stage in stages.items():
        require_refs(stage.get("requiredControls"), set(controls), f"{stage_id}.requiredControls")
        decisions_allowed = stage.get("allowedDecisions")
        if not isinstance(decisions_allowed, list) or len(decisions_allowed) < 2:
            error(f"{stage_id}: requires at least two allowed decisions")

    for test_id, test in tests.items():
        if test.get("plannedGate") not in stages:
            error(f"{test_id}: unknown plannedGate {test.get('plannedGate')!r}")

    links = trace.get("links", [])
    link_index = index(links, "requirementId", "traceability.links", r"REQ-[FQ]-[0-9]{3}")
    missing_links = set(requirements) - set(link_index)
    extra_links = set(link_index) - set(requirements)
    if missing_links:
        error("traceability: requirements without link: " + ", ".join(sorted(missing_links)))
    if extra_links:
        error("traceability: links to unknown requirements: " + ", ".join(sorted(extra_links)))

    linked_tests: set[str] = set()
    linked_components: set[str] = set()
    linked_invariants: set[str] = set()
    for requirement_id, link in link_index.items():
        require_refs(link.get("components"), set(components), f"{requirement_id}.components")
        require_refs(link.get("architecture"), set(invariants), f"{requirement_id}.architecture")
        require_refs(link.get("tests"), set(tests), f"{requirement_id}.tests")
        require_refs(link.get("gates"), set(stages), f"{requirement_id}.gates")
        linked_tests.update(link.get("tests", []))
        linked_components.update(link.get("components", []))
        linked_invariants.update(link.get("architecture", []))

    for label, unlinked in {
        "tests": set(tests) - linked_tests,
        "components": set(components) - linked_components,
        "invariants": set(invariants) - linked_invariants,
    }.items():
        if unlinked:
            error(f"traceability: unlinked {label}: " + ", ".join(sorted(unlinked)))

    for risk_id, risk in risks.items():
        require_refs(risk.get("requirements"), set(requirements), f"{risk_id}.requirements")
        require_refs(risk.get("controls"), set(controls), f"{risk_id}.controls")
        require_refs(risk.get("tests"), set(tests), f"{risk_id}.tests")
        if not risk.get("ownerRole") or not risk.get("treatment"):
            error(f"{risk_id}: missing owner or treatment")

    required_roles = {control.get("ownerRole") for control in controls.values()}
    declared_roles = {item.get("role") for item in roles_doc.get("roles", [])}
    if not required_roles.issubset(declared_roles):
        error("roles: control owners not declared: " + ", ".join(sorted(required_roles - declared_roles)))
    required_assignments = [item for item in roles_doc.get("roles", []) if item.get("requiredForG0")]
    if any(not item.get("assignee") for item in required_assignments):
        error("roles: every role required for G0 must have an operational assignee")

    for decision_id, decision in decisions.items():
        if decision.get("status") not in {"open", "accepted", "rejected", "superseded"}:
            error(f"{decision_id}: invalid status")
        for gate in decision.get("blocks", []):
            if gate not in stages:
                error(f"{decision_id}: unknown blocked gate {gate}")

    for tool_id, tool in tools.items():
        if tool.get("status") not in {"candidate", "observing", "selected", "blocking", "retired"}:
            error(f"{tool_id}: invalid status")
        if not all(tool.get(field) for field in ("purpose", "pinPolicy", "ownerRole", "source")):
            error(f"{tool_id}: purpose, pinPolicy, ownerRole and source are required")
        if tool.get("ownerRole") not in declared_roles:
            error(f"{tool_id}: unknown owner role {tool.get('ownerRole')!r}")
        if not str(tool.get("source", "")).startswith("https://"):
            error(f"{tool_id}: source must be HTTPS")

    assessment_summaries: list[str] = []
    for path in sorted((QUALITY / "assessments").glob("*.json")):
        record = load(str(path.relative_to(ROOT)).replace("\\", "/"))
        record_id = str(record.get("recordId", path.name))
        gate_id = record.get("gateId")
        if gate_id not in stages:
            error(f"{record_id}: unknown gate {gate_id}")
            continue
        if record.get("planVersion") != plan_version:
            error(f"{record_id}: planVersion mismatch")
        decision = record.get("decision")
        if decision not in stages[gate_id].get("allowedDecisions", []):
            error(f"{record_id}: decision {decision!r} not allowed for {gate_id}")

        results = index(record.get("controls", []), "id", f"{record_id}.controls")
        required = set(stages[gate_id].get("requiredControls", []))
        if set(results) != required:
            error(f"{record_id}: controls must exactly match required set for {gate_id}")
        passed = 0
        evidence_total = 0
        evidence_valid = 0
        for control_id, result in results.items():
            outcome = result.get("result")
            if outcome not in {"PASS", "FAIL", "BLOCKED", "NOT_APPLICABLE"}:
                error(f"{record_id}/{control_id}: invalid result {outcome!r}")
            if outcome == "PASS":
                passed += 1
            evidence = result.get("evidence", [])
            if not isinstance(evidence, list) or not evidence:
                error(f"{record_id}/{control_id}: evidence list required")
            for relative in evidence:
                evidence_total += 1
                before = len(ERRORS)
                safe_evidence_path(relative, record_id)
                if len(ERRORS) == before:
                    evidence_valid += 1
        all_pass = len(results) == len(required) and passed == len(required)
        if decision in {"GO", "MERGE", "RC", "RELEASE", "CONTINUE", "RETIRE"} and not all_pass:
            error(f"{record_id}: positive decision with non-passing controls")
        if all_pass and not record.get("approver"):
            error(f"{record_id}: passing record requires an approver")
        open_blockers = [d for d in decisions.values() if d.get("status") == "open" and gate_id in d.get("blocks", [])]
        if open_blockers and decision in {"GO", "MERGE", "RC", "RELEASE", "CONTINUE", "RETIRE"}:
            error(f"{record_id}: positive decision while blocking decisions remain open")
        if gate_id in {"G0", "G5"} and roles_doc.get("independenceFeasible") is not True and decision in {"GO", "RELEASE"}:
            error(f"{record_id}: independent {gate_id} approval is not feasible under the current role model")
        control_completion = passed / len(required) if required else 0.0
        evidence_completion = evidence_valid / evidence_total if evidence_total else 0.0
        traceability_completion = 1.0 if not missing_links and not extra_links else 0.0
        score = 100 * (0.40 * control_completion + 0.35 * evidence_completion + 0.25 * traceability_completion)
        assessment_summaries.append(f"{record_id}: {decision}, completion={score:.1f}/100")

    required_docs = [
        "README.md",
        "CHANGELOG.md",
        "CITATION.cff",
        "CODE_OF_CONDUCT.md",
        "CONTRIBUTING.md",
        "GOVERNANCE.md",
        "SECURITY.md",
        "VERSIONING.md",
        "docs/README.md",
        "docs/development/LOCAL-DEVELOPMENT.md",
        "docs/development/RELEASING.md",
        "fuzz/README.md",
        "docs/quality/SOFTWARE-QUALITY-PLAN.md",
        "docs/quality/STAGE-GATES.md",
        "docs/quality/QUALITY-METRICS.md",
        "docs/architecture/ARCHITECTURE.md",
        "docs/process/DEVELOPMENT-LIFECYCLE.md",
        "docs/process/ROLES-RACI.md",
        "docs/process/CHANGE-CONTROL.md",
        "docs/requirements/REQUIREMENTS.md",
        "docs/github/REPOSITORY-SETTINGS.md",
        "docs/decisions/OPEN-DECISIONS.md",
        "docs/quality/AUDIT-MODEL.md",
        "docs/github/CI-PIPELINE.md",
        "docs/process/DELIVERY-ROADMAP.md",
        "docs/security/THREAT-MODEL.md",
        "docs/research/RESEARCH-TRACE.md",
        "docs/audit/INITIAL-AUDIT-2026-08-13.md",
    ]
    for relative in required_docs:
        if not (ROOT / relative).is_file():
            error(f"required document missing: {relative}")

    if ERRORS:
        print("QUALITY SYSTEM: FAIL")
        for item in ERRORS:
            print(f"- {item}")
        return 1

    print("QUALITY SYSTEM: PASS")
    print(f"- {len(requirements)} requirements with complete bidirectional links")
    print(f"- {len(invariants)} architecture invariants and {len(components)} components")
    print(f"- {len(controls)} controls across {len(stages)} stage gates")
    print(f"- {len(risks)} risks linked to requirements, controls and tests")
    print(f"- {len(tools)} tools registered with pin and qualification policy")
    for summary in assessment_summaries:
        print(f"- {summary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
