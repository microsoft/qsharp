# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Per-commit outcome matrices and bounded, predicate-preserving line reduction."""

from pathlib import Path
import tempfile

from harness.builds import build_revision
from harness.cases import load_cases
from harness.common import ReviewError, git, is_ancestor, preflight, resolve_ref, write_json
from harness.runner import compare, endpoint, run_case


def attribute(config, cases_path, case_id, output, refs=None, mode="release"):
    tools = preflight(config)
    baseline, candidate = config["refs"]["reference"], config["refs"]["candidate"]
    if refs:
        revisions = [baseline] + [resolve_ref(config["repository"], ref) for ref in refs]
    else:
        if not is_ancestor(config["repository"], baseline, candidate):
            raise ReviewError("Attribution across non-ancestral refs requires an explicit ordered --refs list.")
        revisions = [baseline] + git(
            config["repository"], "rev-list", "--reverse", "--topo-order", f"{baseline}..{candidate}"
        ).splitlines()
    revisions = list(dict.fromkeys(revisions))
    work = Path(config["work_dir"])
    work.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="attribute-", dir=work))
    cases, _ = load_cases(cases_path, config, directory / "inputs")
    case = next((case for case in cases if case["id"] == case_id), None)
    if case is None:
        raise ReviewError(f"Case not found or excluded: {case_id}")
    rows, reference = [], None
    for sha in revisions:
        build = build_revision(config, sha, mode, tools)
        result = run_case(config, build, case, directory / sha)
        if reference is None:
            reference = result
        row = {
            "sha": sha,
            "parents": git(config["repository"], "show", "-s", "--format=%P", sha).split(),
            "outcomes": {p: endpoint(case, result, p) for p in case["profiles"]},
            "comparison": compare(case, reference, result),
            "result": result,
        }
        rows.append(row)
        write_json(output, {"case": case, "tools": tools, "rows": rows, "complete": False})
        print(f"{sha[:12]}: {row['outcomes']}", flush=True)
    write_json(output, {"case": case, "tools": tools, "rows": rows, "complete": True})
    return rows


def ddmin(parts, test):
    granularity = 2
    while len(parts) >= 2:
        size = (len(parts) + granularity - 1) // granularity
        for start in range(0, len(parts), size):
            candidate = parts[:start] + parts[start + size :]
            if candidate and test(candidate):
                parts = candidate
                granularity = max(2, granularity - 1)
                break
        else:
            if granularity >= len(parts):
                break
            granularity = min(len(parts), granularity * 2)
    return parts


def reduce_case(config, cases_path, case_id, output, kind, profile, needle, attempts, preserve_regression):
    if kind in ("panic", "rejected") and not needle:
        raise ReviewError("Panic/rejection reduction requires --needle to preserve the failure signature.")
    tools = preflight(config)
    work = Path(config["work_dir"])
    work.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="reduce-", dir=work))
    cases, _ = load_cases(cases_path, config, directory / "inputs")
    original = next((case for case in cases if case["id"] == case_id), None)
    if original is None or "source" not in original or any(k in original for k in ("project_root", "adapter")):
        raise ReviewError("Reduction requires a standalone source case; minimize projects/adapters manually first.")
    if profile not in original["profiles"]:
        raise ReviewError(f"Case does not select profile {profile}")
    case = original | {"profiles": [profile]}
    builds = {
        role: build_revision(config, config["refs"][role], "release", tools)
        for role in ("reference", "candidate")
    }
    cache, tested = {}, 0

    def predicate(source):
        nonlocal tested
        if source in cache:
            return cache[source]
        if tested >= attempts:
            return False
        tested += 1
        current = case | {"source": source}
        candidate = run_case(config, builds["candidate"], current, directory / str(tested) / "candidate")
        matched = endpoint(current, candidate, profile) == kind
        if needle:
            row = candidate["profiles"][profile]
            messages = str(row["compile"].get("message", "")) + row["compile"].get("stderr", "")
            matched = matched and needle in messages
        if matched and preserve_regression:
            reference = run_case(config, builds["reference"], current, directory / str(tested) / "reference")
            matched = compare(current, reference, candidate)[0]["verdict"] == "regression_candidate"
        cache[source] = matched
        return matched

    if not predicate(case["source"]):
        raise ReviewError("Input does not satisfy the requested failure/regression predicate.")
    lines = ddmin(case["source"].splitlines(keepends=True), lambda parts: predicate("".join(parts)))
    reduced = "".join(lines)
    clean = dict(original)
    clean.update(source=reduced, profiles=[profile])
    write_json(output, [clean])
    write_json(
        Path(output).with_suffix(".reduction.json"),
        {
            "attempts": tested,
            "budget_exhausted": tested >= attempts,
            "kind": kind,
            "profile": profile,
            "needle": needle,
            "preserve_regression": preserve_regression,
            "reference": config["refs"]["reference"],
            "candidate": config["refs"]["candidate"],
            "original_lines": len(case["source"].splitlines()),
            "reduced_lines": len(lines),
        },
    )
    return tested, len(lines)
