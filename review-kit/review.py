#!/usr/bin/env python3
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Entry point for the review kit; no third-party Python dependencies."""

import argparse
import json
from pathlib import Path
import sys

from harness.builds import build_revision
from harness.common import ReviewError, is_ancestor, load_config, preflight, read_json
from harness.discovery import discover_samples, snapshot_issues
from harness.investigate import attribute, reduce_case
from harness.report import generate_report
from harness.runner import create_run


def parser():
    cli = argparse.ArgumentParser(description=__doc__)
    commands = cli.add_subparsers(dest="command", required=True)
    for name in ("check", "build", "run", "samples", "issues", "attribute", "reduce"):
        sub = commands.add_parser(name)
        sub.add_argument("config", type=Path)
        if name in ("run", "attribute", "reduce"):
            sub.add_argument("cases", type=Path)
        if name in ("build", "run", "attribute"):
            sub.add_argument("--mode", choices=("release", "debug"), default="release")
        if name in ("samples", "issues", "attribute", "reduce"):
            sub.add_argument("--output", type=Path, required=True)
        if name in ("attribute", "reduce"):
            sub.add_argument("--case", required=True)
        if name == "samples":
            sub.add_argument("--revision", choices=("candidate", "reference"), default="candidate")
        if name == "attribute":
            sub.add_argument("--refs", nargs="+")
        if name == "reduce":
            sub.add_argument("--kind", choices=("panic", "rejected", "wrong_result", "compiler_crash"), required=True)
            sub.add_argument("--profile", choices=("Base", "Adaptive_RI", "Adaptive_RIF", "Adaptive"), required=True)
            sub.add_argument("--needle", default="")
            sub.add_argument("--attempts", type=int, default=100)
            sub.add_argument("--allow-preexisting", action="store_true")
    report = commands.add_parser("report")
    report.add_argument("run", type=Path)
    report.add_argument("--output", type=Path)
    report.add_argument("--overwrite", action="store_true")
    return cli


def main():
    args = parser().parse_args()
    try:
        if args.command == "report":
            manifest = read_json(args.run / "manifest.json")
            destination = args.output or Path(manifest["config"]["report"])
            generate_report(args.run, destination, args.overwrite)
            print(destination)
            return
        config = load_config(args.config)
        if args.command == "check":
            print(json.dumps({"config": config, "tools": preflight(config)}, indent=2))
            for older in ("reference", "previously_reviewed"):
                if older in config["refs"] and not is_ancestor(
                    config["repository"], config["refs"][older], config["refs"]["candidate"]
                ):
                    print(f"WARNING: {older} is not an ancestor of candidate; inspect the rebase/comparison.", file=sys.stderr)
            if config["refs"]["reference"] == config["refs"]["candidate"]:
                print("WARNING: identical endpoints; this is a smoke test, not a regression review.", file=sys.stderr)
            if config["checkout_status"]:
                print("WARNING: main checkout is dirty; pinned builds exclude those changes.", file=sys.stderr)
            for client in config["clients"]:
                if client["status"]:
                    print(f"WARNING: client {client['name']} is dirty; case snapshots include local edits.", file=sys.stderr)
        elif args.command == "build":
            tools = preflight(config)
            for role, sha in config["refs"].items():
                if role != "previously_reviewed":
                    build = build_revision(config, sha, args.mode, tools)
                    print(f"{role}: {build['overlay']}")
        elif args.command == "run":
            if config["debug_validation"] == "all" and args.mode == "release":
                release = create_run(config, args.cases, "release")
                debug = create_run(config, args.cases, "debug")
                print(f"Release: {release}\nDebug: {debug}")
                runs = [release, debug]
            else:
                run = create_run(config, args.cases, args.mode)
                print(run)
                runs = [run]
            if any(read_json(run / "manifest.json")["status"] != "complete" for run in runs):
                sys.exit(2)
        elif args.command == "samples":
            found, pending = discover_samples(config, args.output, args.revision)
            print(f"{found} candidates, {pending} pending manual curation; {args.output}")
        elif args.command == "issues":
            print(f"{snapshot_issues(config, args.output)} issues snapshotted; {args.output}")
        elif args.command == "attribute":
            attribute(config, args.cases, args.case, args.output, args.refs, args.mode)
        elif args.command == "reduce":
            if args.attempts < 1:
                raise ReviewError("--attempts must be positive")
            tested, lines = reduce_case(
                config, args.cases, args.case, args.output, args.kind, args.profile,
                args.needle, args.attempts, not args.allow_preexisting,
            )
            print(f"{tested} predicate attempts; {lines} lines; {args.output}")
    except (ReviewError, OSError, json.JSONDecodeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
