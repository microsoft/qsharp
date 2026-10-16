# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Build case-manifest subsets without breaking manifest-relative paths."""

import fnmatch
import os
from pathlib import Path

from harness.common import KIT, ReviewError, read_json, validate, write_json

PATH_FIELDS = ("source_file", "project_root", "adapter")


def rebase_paths(case, source_manifest, output):
    """Rewrite manifest-relative paths so they resolve from the output manifest's directory."""
    case = dict(case)
    source_dir = Path(source_manifest).resolve().parent
    output_dir = Path(output).resolve().parent
    for field in PATH_FIELDS:
        if field in case:
            absolute = (source_dir / case[field]).resolve()
            if not absolute.exists():
                raise ReviewError(f"{case['id']}: {field} does not exist: {absolute}")
            case[field] = os.path.relpath(absolute, output_dir)
    return case


def subset_manifests(manifests, output, ids=(), patterns=()):
    """Select cases by exact ID or glob across manifests; IDs must stay unique."""
    if not ids and not patterns:
        raise ReviewError("Select cases with --ids and/or --match")
    schema = read_json(KIT / "cases.schema.json")
    wanted, selected, seen = set(ids), [], {}
    for manifest in manifests:
        cases = read_json(manifest)
        validate(cases, schema, str(manifest))
        for case in cases:
            if case["id"] in wanted or any(fnmatch.fnmatch(case["id"], p) for p in patterns):
                if case["id"] in seen:
                    raise ReviewError(f"Case {case['id']} appears in both {seen[case['id']]} and {manifest}")
                seen[case["id"]] = manifest
                selected.append(rebase_paths(case, manifest, output))
    missing = sorted(wanted - set(seen))
    if missing:
        raise ReviewError(f"Case IDs not found: {', '.join(missing[:20])}")
    if not selected:
        raise ReviewError("No cases matched")
    validate(selected, schema, "subset")
    write_json(output, selected)
    return len(selected)


def export_run_cases(run, ids, output):
    """Copy the original (pre-snapshot) case entries of a run into a new manifest."""
    manifest = read_json(Path(run) / "manifest.json")
    source = Path(manifest["cases_path"])
    if not source.is_file():
        raise ReviewError(f"Original case manifest is gone: {source}")
    return subset_manifests([source], output, ids=ids)
