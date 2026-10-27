"""The acceptance tests' results from `cargo test`, as the og plugin's run collector reads them.

The test cases run as `cargo test` against the fakes (`DEC-65`); `collect_run.py` reads a
Playwright JSON report. This turns cargo's output into one, without deciding anything: a test
whose name starts `tc_NN_MM_` is test case `TC-NN-MM`; its status is cargo's; its steps are the
lines it printed with `og-step <ID> <n> — <what>` before carrying each step out, so a step it never
reached is missing. Run from the repository's root:

    cargo test --workspace tc_10_0 -- --show-output > "$RUN_DIR/cargo-test.txt" 2>&1
    python3 scripts/cargo_report.py "$RUN_DIR/cargo-test.txt" > "$RUN_DIR/playwright-report.json"
"""

import json
import re
import sys

RESULT = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored)")
OUTPUT = re.compile(r"^---- (\S+) stdout ----$")
STEP = re.compile(r"og-step (TC-\d{2}-\d{2}) (\d+) — (.*)$")
CASE = re.compile(r"(?:^|::)tc_(\d{2})_(\d{2})_(\w+)$")
STATUS = {"ok": "passed", "FAILED": "failed", "ignored": "skipped"}


def main(path):
    lines = open(path, encoding="utf-8").read().splitlines()
    results, printed, current = {}, {}, None
    for line in lines:
        if m := RESULT.match(line):
            results[m.group(1)] = STATUS[m.group(2)]
        if m := OUTPUT.match(line):
            current = m.group(1)
            printed[current] = []
        elif current and (line in ("successes:", "failures:") or line.startswith("test result:")):
            current = None
        elif current:
            printed[current].append(line)
    specs = []
    for name, status in results.items():
        case = CASE.search(name)
        if not case:
            continue
        output = printed.get(name, [])
        steps = [m for m in map(STEP.search, output) if m]
        error = "\n".join(l for l in output if not STEP.search(l)).strip()
        json_steps = []
        for i, step in enumerate(steps):
            failed = status == "failed" and i == len(steps) - 1
            json_steps.append({
                "title": f"{step.group(2)} — {step.group(3)}",
                "category": "test.step",
                "duration": 0,
                **({"error": {"message": error}} if failed else {}),
            })
        specs.append({
            "title": f"TC-{case.group(1)}-{case.group(2)} {case.group(3).replace('_', ' ')}",
            "file": name,
            "tests": [{
                "projectName": "desktop",
                "results": [{
                    "status": status,
                    "duration": 0,
                    "steps": json_steps,
                    "errors": [{"message": error}] if status == "failed" else [],
                }],
            }],
        })
    json.dump({"suites": [{"title": "cargo test", "specs": specs}]}, sys.stdout, ensure_ascii=False, indent=2)


if __name__ == "__main__":
    main(sys.argv[1])
