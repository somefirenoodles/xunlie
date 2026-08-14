#!/usr/bin/env python3
"""Create a fail-closed draft GateRecord for a stage."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gate", choices=[f"G{i}" for i in range(8)])
    parser.add_argument("candidate", help="Commit, tag or artifact digest")
    parser.add_argument("--output", type=Path, help="Output path; stdout when omitted")
    args = parser.parse_args()

    with (ROOT / "quality/stages.json").open(encoding="utf-8") as handle:
        stages = json.load(handle)
    stage = next(item for item in stages["stages"] if item["id"] == args.gate)
    now = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    record = {
        "schemaVersion": "xunlie.quality.gate-record/v1",
        "recordId": f"GATE-{args.gate}-{now[:10]}-DRAFT",
        "planVersion": stages["planVersion"],
        "gateId": args.gate,
        "candidate": args.candidate,
        "decision": "BLOCKED",
        "controls": [
            {"id": control_id, "result": "BLOCKED", "evidence": ["REPLACE-WITH-EVIDENCE-PATH"], "reason": "Not evaluated"}
            for control_id in stage["requiredControls"]
        ],
        "waivers": [],
        "approver": None,
        "createdAt": now,
    }
    rendered = json.dumps(record, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

