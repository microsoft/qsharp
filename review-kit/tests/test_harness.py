# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""Fast self-tests; no compiler build, installed qdk, or network is required."""

import json
import io
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from harness.analysis import (  # noqa: E402
    callable_stages, compare_runs, hierarchy_violations, match_issues, normalize_signature,
    parse_selection, select_cases, summarize_run,
)
from harness.builds import refresh_timestamps, verify_build  # noqa: E402
from harness.cases import load_cases, snapshot_project  # noqa: E402
from harness.common import (  # noqa: E402
    KIT, ReviewError, isolated, load_config, read_json, tree_digest, validate, write_json,
)
from harness.discovery import infer_entry, snapshot_issues  # noqa: E402
from harness.execution import enclosing_function, last_exception, parse_panics  # noqa: E402
from harness.extraction import convert_dump, insert_at_function_start, instrument  # noqa: E402
from harness.investigate import (  # noqa: E402
    adapter_variants, ddmin, qsharp_constants, render_adapter, replace_constant,
)
from harness.manifests import export_run_cases, subset_manifests  # noqa: E402
from harness.records import encode, parse_output, validate_value  # noqa: E402
from harness.report import generate_report, source_repro  # noqa: E402
from harness.runner import compare, create_run, endpoint, prune_qir, run_case  # noqa: E402
from harness.statecheck import check as statecheck  # noqa: E402
from harness import worker  # noqa: E402

sys.path.insert(0, str(KIT / "generators"))
import callable_chains  # noqa: E402


ONE = {"type": "result", "value": 1}
ZERO = {"type": "result", "value": 0}
CASE = {
    "id": "test",
    "source": "operation Main() : Result { Zero }",
    "entry": "Main()",
    "profiles": ["Base"],
    "shots": 2,
    "seed": 1,
    "oracle": {"kind": "deterministic"},
}


def good(value=ONE):
    return {
        "simulator": {"status": "ok", "values": [value, value]},
        "profiles": {
            "Base": {
                "compile": {"status": "ok"},
                "runner": {"status": "ok", "values": [value, value]},
            }
        },
    }


def frame(*records):
    return "START\nMETADATA\tentry_point\n" + "\n".join(records) + "\nEND\t0\n"


class ConfigurationTests(unittest.TestCase):
    def test_minimal_config_and_relative_defaults(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "review.json"
            write_json(path, {"reference": "old", "candidate": "new", "report": "report.md"})
            with patch("harness.common.git", return_value=directory), patch(
                "harness.common.resolve_ref", side_effect=lambda repo, ref: ref + "-sha"
            ):
                config = load_config(path)
            self.assertEqual(config["refs"], {"reference": "old-sha", "candidate": "new-sha"})
            self.assertEqual(config["venv"], str(Path(directory) / ".venv"))
            self.assertEqual(config["report"], str(Path(directory) / "report.md"))

    def test_unknown_field(self):
        with self.assertRaisesRegex(ReviewError, "unknown"):
            validate({"reference": "a", "candidate": "b", "profiels": []}, read_json(KIT / "review.schema.json"))

    def test_bool_is_not_a_number(self):
        with self.assertRaisesRegex(ReviewError, "integer"):
            validate({"reference": "a", "candidate": "b", "limits": {"shots": True}}, read_json(KIT / "review.schema.json"))

    def test_duplicate_json_keys(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.json"
            path.write_text('{"candidate": "a", "candidate": "b"}')
            with self.assertRaisesRegex(ReviewError, "Duplicate"):
                read_json(path)

    def test_cases_require_explicit_oracle(self):
        with self.assertRaisesRegex(ReviewError, "oracle"):
            validate([{"id": "x", "source": "", "entry": "Main()"}], read_json(KIT / "cases.schema.json"))

    def test_duplicate_case_ids(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cases.json"
            write_json(path, [CASE, CASE])
            with self.assertRaisesRegex(ReviewError, "duplicate"):
                load_cases(path, {"input": {}, "profiles": ["Base"], "limits": {"shots": 2, "seed": 1}}, directory)

    def test_exclusions_are_recorded(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cases.json"
            write_json(path, [CASE])
            config = {"input": {"exclusions": [{"match": "test", "reason": "budget"}]}}
            cases, skipped = load_cases(path, config, directory)
            self.assertEqual(cases, [])
            self.assertEqual(skipped, [{"id": "test", "reason": "budget"}])


class RecordTests(unittest.TestCase):
    def test_nested_values_and_shot_multiplicity(self):
        text = frame(
            "OUTPUT\tTUPLE\t5\troot",
            "OUTPUT\tINT\t42\ti",
            "OUTPUT\tBOOL\ttrue\tb",
            "OUTPUT\tDOUBLE\t-0.0\td",
            "OUTPUT\tARRAY\t1\ta",
            "OUTPUT\tRESULT\t1\tr",
            "OUTPUT\tTUPLE\t0\tu",
        )
        parsed = parse_output(text + text, 2)
        expected = encode((42, True, -0.0, [1], None))
        expected["items"][3]["items"][0] = ONE
        self.assertEqual(parsed, [expected, expected])
        self.assertNotEqual(encode(0.0), encode(-0.0))

    def test_array_and_tuple_remain_distinct(self):
        self.assertNotEqual(encode([1]), encode((1,)))

    def test_nan_and_infinity(self):
        self.assertEqual(parse_output(frame("OUTPUT\tDOUBLE\tNaN"), 1), [encode(float("nan"))])
        self.assertEqual(parse_output(frame("OUTPUT\tDOUBLE\tinf"), 1), [encode(float("inf"))])

    def test_invalid_protocols(self):
        invalid = [
            "",
            "START\nEND\t0\n",
            "START\nSTART\n",
            frame("OUTPUT\tARRAY\t2", "OUTPUT\tINT\t1"),
            frame("OUTPUT\tARRAY\t-1"),
            frame("OUTPUT\tINT\t1", "OUTPUT\tINT\t2"),
            frame("OUTPUT\tBOOL\t1"),
            frame("OUTPUT\tRESULT\t2"),
            frame("OUTPUT\tOTHER\t1"),
            frame("OUTPUT\tINT\t1").replace("END\t0", "END\t1"),
            frame("OUTPUT\tINT\t1") + "trailing garbage\n",
        ]
        for text in invalid:
            with self.subTest(text=text), self.assertRaises(ReviewError):
                parse_output(text, 1)

    def test_wrong_shot_count(self):
        with self.assertRaisesRegex(ReviewError, "expected 2"):
            parse_output(frame("OUTPUT\tINT\t1"), 2)

    def test_expected_type_validation(self):
        for value in (ONE, encode((1, False, -0.0)), encode(float("nan"))):
            validate_value(value)
        for value in ({"type": "result", "value": True}, {"type": "double", "value": "-0.0"},
                      {"type": "tuple", "items": []}, {"type": "unit", "value": None}):
            with self.subTest(value=value), self.assertRaises(ReviewError):
                validate_value(value)
        with self.assertRaises(ReviewError):
            validate_value({"type": "double", "value": "0x1p+999999999999"})


class ClassificationTests(unittest.TestCase):
    def test_wrong_result_regression(self):
        candidate = good()
        candidate["profiles"]["Base"]["runner"]["values"] = [ZERO, ZERO]
        self.assertEqual(compare(CASE, good(), candidate)[0]["verdict"], "regression_candidate")

    def test_validated_fix(self):
        reference = good()
        reference["profiles"]["Base"]["compile"] = {"status": "panic", "message": "panic"}
        self.assertEqual(compare(CASE, reference, good())[0]["verdict"], "fix_candidate")

    def test_acceptance_is_not_a_fix(self):
        case = CASE | {"oracle": {"kind": "compile_only"}}
        reference = good()
        reference["profiles"]["Base"]["compile"] = {"status": "diagnostic"}
        self.assertEqual(compare(case, reference, good())[0]["verdict"], "newly_accepted_unvalidated")

    def test_stochastic_observations_not_miscompile(self):
        case = CASE | {"oracle": {"kind": "observe"}}
        candidate = good(ZERO)
        self.assertEqual(compare(case, good(), candidate)[0]["verdict"], "inconclusive")

    def test_changed_simulator_without_expected_is_ambiguous(self):
        self.assertEqual(compare(CASE, good(), good(ZERO))[0]["verdict"], "inconclusive")

    def test_unit_is_not_an_oracle(self):
        unit = {"type": "unit"}
        self.assertEqual(endpoint(CASE, good(unit), "Base"), "unit_unvalidated")

    def test_runner_failure_not_success(self):
        candidate = good()
        candidate["profiles"]["Base"]["runner"] = {"status": "runner_error", "returncode": 1}
        self.assertEqual(endpoint(CASE, candidate, "Base"), "execution_failure")

    def test_parser_failure_is_infrastructure(self):
        candidate = good()
        candidate["profiles"]["Base"]["runner"] = {"status": "parser_error"}
        self.assertEqual(endpoint(CASE, candidate, "Base"), "parser_error")
        self.assertEqual(compare(CASE, good(), candidate)[0]["verdict"], "inconclusive")

    def test_explicit_expected_detects_shared_wrong_answer(self):
        case = CASE | {"oracle": {"kind": "deterministic", "expected": ONE}}
        self.assertEqual(endpoint(case, good(ZERO), "Base"), "wrong_result")

    def test_simulator_panic_is_not_hidden_by_successful_qir(self):
        candidate = good()
        candidate["simulator"] = {"status": "panic", "message": "native panic"}
        self.assertEqual(compare(CASE, good(), candidate)[-1]["verdict"], "simulator_regression_candidate")

    def test_other_profiles_continue_after_panic(self):
        case = CASE | {"profiles": ["Base", "Adaptive"]}
        config = {"keep_qir": False}
        build = {"sha": "sha", "mode": "release"}
        calls = []

        def fake_stage(config, build, case, name, profile=None, qir_path=None):
            calls.append((name, profile))
            if name == "simulate":
                return {"status": "timeout"}
            if name == "compile":
                return {"status": "panic"} if profile == "Base" else {"status": "ok", "qir": "qir"}
            return {"status": "ok", "values": [ONE, ONE]}

        with tempfile.TemporaryDirectory() as directory, patch("harness.runner.stage", side_effect=fake_stage):
            result = run_case(config, build, case, directory)
        self.assertEqual(result["profiles"]["Base"]["compile"]["status"], "panic")
        self.assertEqual(result["profiles"]["Adaptive"]["runner"]["status"], "ok")
        self.assertIn(("compile", "Adaptive"), calls)

    def test_historical_failure_qir_is_kept(self):
        with tempfile.TemporaryDirectory() as directory:
            results = {"reference": good(), "candidate": good(), "historical_reference": good()}
            results["historical_reference"]["profiles"]["Base"]["runner"]["values"] = [ZERO, ZERO]
            for name, result in results.items():
                path = Path(directory) / f"{name}.ll"
                path.write_text("raw")
                result["profiles"]["Base"]["compile"]["qir_path"] = str(path)
            prune_qir({"keep_qir": False}, CASE, results)
            self.assertTrue((Path(directory) / "historical_reference.ll").exists())


class ProcessTests(unittest.TestCase):
    def call(self, code, timeout=3):
        with tempfile.TemporaryDirectory() as directory:
            return isolated(
                [sys.executable, "-c", code], payload={}, env=os.environ.copy(),
                cwd=directory, timeout=timeout,
            )

    def test_timeout(self):
        self.assertEqual(self.call("import time; time.sleep(20)", timeout=0.2)["status"], "timeout")

    def test_signal(self):
        self.assertEqual(self.call("import os,signal; os.kill(os.getpid(), signal.SIGKILL)")["status"], "crash")

    def test_missing_json(self):
        self.assertEqual(self.call("print('not json')")["status"], "infrastructure_error")

    def test_nonzero_with_success_shaped_stdout(self):
        result = self.call('import sys; print(\'{"status":"ok"}\'); sys.exit(1)')
        self.assertEqual(result["status"], "infrastructure_error")

    def test_stderr_preserved(self):
        result = self.call('import sys; print("stage",file=sys.stderr); print(\'{"status":"panic"}\')')
        self.assertEqual(result["status"], "panic")
        self.assertIn("stage", result["stderr"])


class WorkerBoundaryTests(unittest.TestCase):
    def call(self, error):
        output = io.StringIO()
        with patch("harness.worker.execute", side_effect=error), patch(
            "sys.stdin", io.StringIO("{}")
        ), patch("sys.stdout", output):
            worker.main()
        return json.loads(output.getvalue())

    def test_panic_exception_is_a_base_exception(self):
        class PanicException(BaseException):
            pass

        self.assertEqual(self.call(PanicException("native panic"))["status"], "panic")

    def test_python_error_is_infrastructure(self):
        result = self.call(ValueError("bad adapter"))
        self.assertEqual(result["status"], "infrastructure_error")
        self.assertIn("bad adapter", result["traceback"])

    def test_interrupt_is_not_swallowed(self):
        with self.assertRaises(KeyboardInterrupt):
            self.call(KeyboardInterrupt())


class HelperTests(unittest.TestCase):
    def test_package_boundaries_are_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            copied = snapshot_project(KIT / "examples/project", Path(directory), {})
            manifest = read_json(copied / "qsharp.json")
            dependency = Path(manifest["dependencies"]["Dependency"]["path"])
            self.assertNotEqual(copied, dependency)
            self.assertTrue((dependency / "src/MeasureOne.qs").exists())
            self.assertFalse((copied / "src/MeasureOne.qs").exists())

    def test_ddmin_preserves_predicate(self):
        self.assertEqual(ddmin(["a", "bug", "b", "c"], lambda parts: "bug" in parts), ["bug"])

    def test_inference_is_conservative(self):
        self.assertEqual(infer_entry("namespace A { @EntryPoint() operation Go() : Unit {} }"), "A.Go()")
        self.assertIsNone(infer_entry("operation Main(x : Int) : Unit {}"))
        self.assertIsNone(infer_entry("namespace A {} namespace B { operation Main() : Unit {} }"))

    def test_issue_selection_includes_closed_and_explicit_older_issues(self):
        config = {
            "repository": "/unused",
            "refs": {"reference": "old", "candidate": "new"},
            "input": {"issues": {"author": "amcasey", "min_number": 3461, "include": [3440]}},
        }
        requests = []

        def api(endpoint, paginated=False):
            requests.append(endpoint)
            if "?creator=" in endpoint:
                return [{"number": 3500}, {"number": 3400}]
            if "/comments?" in endpoint:
                return []
            number = int(endpoint.rsplit("/", 1)[1])
            return {"number": number, "html_url": "url", "title": "title", "state": "closed", "body": "body", "updated_at": "date"}

        with tempfile.TemporaryDirectory() as directory, patch("harness.discovery._api", side_effect=api), patch(
            "harness.discovery.git", return_value="Fix #3600"
        ):
            path = Path(directory) / "issues.json"
            snapshot_issues(config, path)
            snapshot = read_json(path)
        self.assertEqual([i["number"] for i in snapshot["issues"]], [3440, 3500, 3600])
        self.assertTrue(any("state=all" in r for r in requests))

    def test_changed_package_cache_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "qdk").mkdir()
            (root / "qdk/file.py").write_text("before")
            build = {"overlay": directory, "tools": {}, "package_digest": tree_digest(root / "qdk")}
            (root / "qdk/file.py").write_text("after")
            with self.assertRaisesRegex(ReviewError, "package changed"):
                verify_build(build, {})

    def test_inline_repro_is_valid_python(self):
        compile(source_repro(CASE), "<repro>", "exec")

    def test_report_handles_incomplete_run_and_protects_existing_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_json(root / "manifest.json", {
                "config": {"refs": {"reference": "old", "candidate": "new"}},
                "mode": "release", "status": "failed", "skipped": [],
            })
            write_json(root / "cases.json", [CASE])
            (root / "results.jsonl").write_text("")
            output = root / "report.md"
            generate_report(root, output)
            self.assertIn("run status: `failed`", output.read_text())
            with self.assertRaisesRegex(ReviewError, "Report exists"):
                generate_report(root, output)


def comparison_row(case_id, statuses, message="", verdicts=None):
    """A results.jsonl row; statuses maps profile -> (reference, candidate)."""
    rows, profiles = [], {}
    for profile, (old, new) in statuses.items():
        verdict = (verdicts or {}).get(profile) or (
            "regression_candidate" if old == "validated" and new != "validated"
            else "fix_candidate" if new == "validated" and old != "validated"
            else "validated_unchanged" if old == new == "validated"
            else "pre_existing_or_changed_failure"
        )
        rows.append({"profile": profile, "reference": old, "candidate": new, "verdict": verdict})
        compiled = {"status": "ok", "seconds": 0.5} if new == "validated" else {"status": "panic", "message": message, "seconds": 0.5}
        profiles[profile] = {"compile": compiled}
    result = {"profiles": profiles}
    return {"case": case_id, "results": {"reference": result, "candidate": result}, "comparison": rows}


def write_run(root, rows, refs=None, cases_path=None):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    write_json(root / "manifest.json", {
        "config": {"refs": refs or {"reference": "a" * 40, "candidate": "b" * 40}},
        "status": "complete", "cases_path": str(cases_path or root / "cases.json"),
    })
    write_json(root / "cases.json", [CASE | {"id": row["case"]} for row in rows])
    (root / "results.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
    return root


class AnalysisTests(unittest.TestCase):
    def test_signature_normalization_groups_ids(self):
        a = normalize_signature("PostArgPromote: Expr 67 passes Expr 70\nsecond line")
        b = normalize_signature("\n  PostArgPromote: Expr 1   passes Expr 2")
        self.assertEqual(a, b)
        self.assertEqual(a, "PostArgPromote: Expr N passes Expr N")

    def test_profile_hierarchy(self):
        statuses = {"Base": "rejected", "Adaptive_RI": "validated", "Adaptive_RIF": "validated", "Adaptive": "rejected"}
        self.assertEqual(
            hierarchy_violations(statuses),
            [("Adaptive_RI", "Adaptive", "rejected"), ("Adaptive_RIF", "Adaptive", "rejected")],
        )
        self.assertEqual(hierarchy_violations({"Adaptive_RIF": "validated", "Adaptive": "validated"}), [])

    def test_summary_tables_signatures_and_new_hierarchy_violations(self):
        with tempfile.TemporaryDirectory() as directory:
            run = write_run(directory, [
                comparison_row("one", {"Adaptive_RIF": ("validated", "validated"), "Adaptive": ("validated", "panic")},
                               "Local 4 is not bound"),
                comparison_row("two", {"Adaptive_RIF": ("panic", "panic"), "Adaptive": ("panic", "panic")},
                               "Local 9 is not bound"),
            ])
            summary = summarize_run(run)
        self.assertEqual(summary["verdicts"]["regression_candidate"], 1)
        self.assertEqual(summary["flagged"][0]["signature"], "Local N is not bound")
        groups = {(g["profile"], g["signature"]): g for g in summary["candidate_failures"]}
        self.assertEqual(groups[("Adaptive", "Local N is not bound")]["count"], 2)
        self.assertEqual(groups[("Adaptive", "Local N is not bound")]["reference"], {"validated": 1, "panic": 1})
        self.assertEqual(summary["hierarchy"]["candidate"], 1)
        self.assertEqual(summary["hierarchy"]["new_at_candidate"][0]["case"], "one")

    def test_run_comparison_selection_and_export(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "cases" / "cases.json"
            (root / "cases" / "adapters").mkdir(parents=True)
            (root / "cases" / "adapters" / "x.py").write_text("def prepare(ctx):\n    return 'Main()', ()\n")
            write_json(manifest, [CASE | {"id": "fixed"}, CASE | {"id": "lost"},
                                  {"id": "py", "adapter": "adapters/x.py", "oracle": {"kind": "compile_only"}}])
            old = write_run(root / "old", [
                comparison_row("fixed", {"Base": ("panic", "panic")}),
                comparison_row("lost", {"Base": ("panic", "validated")}),
            ], cases_path=manifest)
            new = write_run(root / "new", [
                comparison_row("fixed", {"Base": ("panic", "validated")}),
                comparison_row("lost", {"Base": ("panic", "panic")}),
            ], cases_path=manifest)
            result = compare_runs(old, new)
            directions = {c["examples"][0]: c["direction"] for c in result["changes"]}
            self.assertEqual(directions, {"fixed": "better", "lost": "worse"})
            self.assertEqual(result["reference_changes"], [])
            chosen = select_cases(old, new, "a=validated,b!=validated")
            self.assertEqual(chosen, ["lost"])
            output = root / "out" / "lost.json"
            export_run_cases(new, chosen, output)
            self.assertEqual([c["id"] for c in read_json(output)], ["lost"])
        with self.assertRaises(ReviewError):
            parse_selection("a==validated")

    def test_callable_stage_classifier(self):
        source = """
            struct Box { F : (Qubit => Unit) }
            function Make(op : (Qubit => Unit)) : (Qubit => Unit) { q => op(q) }
            operation Main() : Unit {
                let f = if true { H } else { X };
                ApplyToEach(Make(f), []);
            }
        """
        self.assertTrue({"factory", "closure", "record", "selection", "library"} <= callable_stages(source))
        self.assertEqual(callable_stages("operation Main() : Unit {}"), set())

    def test_issue_matching_uses_stable_prefix(self):
        snapshot = {"issues": [
            {"number": 1, "state": "open", "title": "[Panic] something",
             "body": "PostArgPromote/PostAll call invariant violation: Expr 3 passes Expr 5"},
            {"number": 2, "state": "closed", "title": "unrelated", "body": "no panic here"},
        ]}
        result = match_issues(snapshot, ["PostArgPromote/PostAll call invariant violation: Expr 67 passes Expr 70"])
        self.assertEqual([i["number"] for i in next(iter(result.values()))["issues"]], [1])


class ManifestTests(unittest.TestCase):
    def test_subset_rebases_relative_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "a" / "adapters").mkdir(parents=True)
            (root / "a" / "adapters" / "x.py").write_text("")
            write_json(root / "a" / "cases.json", [
                {"id": "py-1", "adapter": "adapters/x.py", "oracle": {"kind": "compile_only"}}, CASE,
            ])
            write_json(root / "b.json", [CASE | {"id": "other"}])
            output = root / "c" / "d" / "subset.json"
            count = subset_manifests([root / "a" / "cases.json", root / "b.json"], output, ids=["other"], patterns=["py-*"])
            cases = read_json(output)
            self.assertEqual(count, 2)
            self.assertEqual((output.parent / cases[0]["adapter"]).resolve(), (root / "a/adapters/x.py").resolve())
            with self.assertRaisesRegex(ReviewError, "not found"):
                subset_manifests([root / "b.json"], output, ids=["missing"])
            with self.assertRaisesRegex(ReviewError, "appears in both"):
                subset_manifests([root / "b.json", root / "b.json"], output, patterns=["*"])


class ExecutionTests(unittest.TestCase):
    STDERR = (
        "thread '<unnamed>' (1234) panicked at source/compiler/qsc_x/src/a.rs:12:9:\n"
        "first message\nnote: run with RUST_BACKTRACE\n"
        "thread '<unnamed>' (1234) panicked at source/compiler/qsc_x/src/a.rs:12:9:\n"
        "first message\n"
        "Traceback (most recent call last):\n  File \"x.py\"\npyo3_runtime.PanicException: first message\n"
    )

    def test_panic_parsing_deduplicates(self):
        panics = parse_panics(self.STDERR)
        self.assertEqual(panics, [{"path": "source/compiler/qsc_x/src/a.rs", "line": 12, "column": 9,
                                   "message": "first message"}])
        self.assertEqual(last_exception(self.STDERR), "pyo3_runtime.PanicException: first message")

    def test_enclosing_function(self):
        text = "impl A {\n    pub(crate) fn outer(\n        x: u8,\n    ) {\n        let y = 1;\n    }\n}\nfn other() {}\n"
        self.assertEqual(enclosing_function(text, 5), "outer")
        self.assertEqual(enclosing_function(text, 8), "other")
        self.assertIsNone(enclosing_function(text, 1))


class ExtractionTests(unittest.TestCase):
    def test_hook_insertion(self):
        text = "fn helper<T: Into<u8>>(source: &str, expected: (u8, u8)) -> Result<(), String> where T: Copy {\n    body();\n}\n"
        patched = insert_at_function_start(text, "helper", "hook();")
        self.assertIn("{\n    hook();\n    body();", patched)
        with self.assertRaisesRegex(ReviewError, "exactly one"):
            insert_at_function_start(text + text, "helper", "hook();")
        with self.assertRaisesRegex(ReviewError, "exactly one"):
            insert_at_function_start(text, "missing", "hook();")

    def test_instrument_edits_owned_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src/utils.rs").write_text("fn check(source: &str) {\n}\n")
            spec = {"helper_file": "src/utils.rs", "helper_code": "\n// helper\n",
                    "hooks": [{"file": "src/utils.rs", "function": "check", "code": "dump(source);"}]}
            self.assertEqual(instrument(root, spec), ["src/utils.rs"])
            self.assertEqual((root / "src/utils.rs").read_text(), "fn check(source: &str) {\n    dump(source);\n}\n\n// helper\n")

    def test_dump_conversion(self):
        main = "@EntryPoint() operation Main() : Int { 1 }"
        lines = [
            json.dumps({"kind": "semantic", "test": "a::b", "source": main, "expected": {"type": "int", "value": 1}}),
            json.dumps({"kind": "semantic", "test": "a::c", "source": main, "expected": {"type": "int", "value": 1}}),
            json.dumps({"kind": "semantic", "test": "a::d", "source": "operation Other() : Unit {}",
                        "expected": {"type": "unit"}}),
            json.dumps({"kind": "semantic", "test": "a::e", "source": "x", "expected": None}),
            json.dumps({"kind": "qir_int", "test": "a::f", "source": main.replace("1 }", "2 }"),
                        "expected": {"type": "tuple", "items": [{"type": "double_bits", "value": 4607182418800017408}]}}),
            json.dumps({"kind": "semantic", "test": "a::g", "source": main.replace("1 }", "3 }"), "expected": {"type": "int", "value": 3}}),
            json.dumps({"kind": "semantic", "test": "a::h", "source": main.replace("1 }", "3 }"), "expected": {"type": "int", "value": 4}}),
        ]
        cases, stats = convert_dump(lines, "c" * 40, "crate")
        self.assertEqual(stats, {"records": 7, "unsupported_value": 1, "no_entry": 1, "conflict": 1})
        self.assertEqual(len(cases), 2)
        doubles = [c for c in cases if c["id"].startswith("t-qir_int-")]
        self.assertEqual(doubles[0]["oracle"]["expected"], {"type": "tuple", "items": [encode(1.0)]})
        validate(cases, read_json(KIT / "cases.schema.json"))


class AdapterReductionTests(unittest.TestCase):
    ADAPTER = (
        'SRC = """\noperation A() : Unit {}\noperation B() : Unit {}\n"""\n\n'
        "def prepare(ctx):\n    ctx.eval(SRC)\n    c = ctx.code\n"
        "    f = c.Wrap(c.Pick([c.A(), c.B()]))\n    return (c.Run, (f,))\n"
    )

    def test_variants_unwrap_calls_and_drop_elements(self):
        variants = list(adapter_variants(render_adapter(__import__("ast").parse(self.ADAPTER))))
        expressions = {line.strip() for v in variants for line in v.splitlines() if line.strip().startswith("f =")}
        self.assertIn("f = c.Pick([c.A(), c.B()])", expressions)
        self.assertIn("f = c.Wrap([c.A(), c.B()])", expressions)
        self.assertIn("f = c.Wrap(c.Pick([c.B()]))", expressions)
        for variant in variants:
            compile(variant, "<variant>", "exec")

    def test_qsharp_constants_round_trip(self):
        self.assertEqual(qsharp_constants(self.ADAPTER), ["SRC"])
        reduced = replace_constant(self.ADAPTER, "SRC", "operation A() : Unit {}\n")
        self.assertIn('SRC = """operation A() : Unit {}\n"""', reduced)
        self.assertNotIn("operation B", reduced)


class GeneratorTests(unittest.TestCase):
    def test_generated_cases_are_valid_and_deterministic(self):
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            cases = callable_chains.generate(first, 12, 8, 6, seed=3, profiles=["Adaptive_RIF"])
            callable_chains.generate(second, 12, 8, 6, seed=3, profiles=["Adaptive_RIF"])
            self.assertEqual((Path(first) / "cases.json").read_text(), (Path(second) / "cases.json").read_text())
            validate(cases, read_json(KIT / "cases.schema.json"))
            loaded, _ = load_cases(Path(first) / "cases.json", {"input": {}, "profiles": ["Base"],
                                                                 "limits": {"shots": 2, "seed": 1}}, first)
            self.assertEqual(len(loaded), 26)
            for case in cases:
                validate_value(case["oracle"]["expected"])
                if "adapter" in case:
                    compile((Path(first) / case["adapter"]).read_text(), case["adapter"], "exec")

    def test_model_inverse_and_composition(self):
        tag = callable_chains.M.tag(5, 1, "c")
        identity = callable_chains.M.ident()
        self.assertEqual(tag.then(tag.adj()).perm, identity.perm)
        self.assertEqual(tag.then(tag.adj()).phase, 0)
        self.assertNotEqual(tag.adj().phase, tag.phase)


class StatecheckTests(unittest.TestCase):
    @staticmethod
    def qir(*calls):
        body = "\n".join(f"  call void @__quantum__qis__{c}" for c in calls)
        return f"define void @main() #0 {{\n{body}\n  ret void\n}}\nattributes #0 = {{ \"required_num_qubits\"=\"2\" }}\n"

    Q1 = "%Qubit* inttoptr (i64 1 to %Qubit*)"

    def test_unmeasured_qubit_left_flipped_is_reported(self):
        q1 = self.Q1
        result = statecheck(self.qir("x__body(%Qubit* null)", f"h__body({q1})", f"s__body({q1})", f"s__body({q1})",
                                     f"h__body({q1})", "mresetz__body(%Qubit* null, %Result* null)"))
        self.assertEqual(result["p1"], [1.0, 1.0])
        self.assertEqual(result["measured"], [0])
        self.assertEqual(result["dirty_unmeasured"], [1])

    def test_adjoint_uncomputes(self):
        q1 = self.Q1
        result = statecheck(self.qir(f"h__body({q1})", f"s__adj({q1})", f"s__body({q1})", f"h__body({q1})"))
        self.assertEqual(result["dirty_unmeasured"], [])

    def test_branching_qir_is_rejected(self):
        with self.assertRaisesRegex(ReviewError, "straight-line"):
            statecheck("br i1 %0, label %a, label %b\n\"required_num_qubits\"=\"1\"")


class ExtraRecordTests(unittest.TestCase):
    def test_strings_are_typed_simulator_values(self):
        self.assertEqual(encode("<Range> 2..1"), {"type": "string", "value": "<Range> 2..1"})
        validate_value({"type": "string", "value": "Foo"})
        with self.assertRaises(ReviewError):
            validate_value({"type": "string", "value": 1})


class BuildFreshnessTests(unittest.TestCase):
    def test_refresh_timestamps_skips_git_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("")
            (root / ".git").write_text("gitdir: elsewhere")
            os.utime(root / "src/lib.rs", (1, 1))
            os.utime(root / ".git", (1, 1))
            refresh_timestamps(root)
            self.assertGreater((root / "src/lib.rs").stat().st_mtime, 1)
            self.assertEqual((root / ".git").stat().st_mtime, 1)


class RunOrderingTests(unittest.TestCase):
    def test_results_are_rewritten_in_manifest_order(self):
        import time

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cases = [CASE | {"id": f"c{i}"} for i in range(4)]
            write_json(root / "cases.json", cases)
            config = {"work_dir": str(root / "work"), "refs": {"reference": "r", "candidate": "c"},
                      "input": {}, "profiles": ["Base"], "limits": {"jobs": 4, "shots": 2, "seed": 1},
                      "keep_qir": False}

            def slow_first(config, build, case, directory):
                time.sleep(0.3 if case["id"] == "c0" else 0)
                return good()

            with patch("harness.runner.preflight", return_value={}), patch(
                "harness.runner.build_revision", return_value={"sha": "x", "mode": "release"}
            ), patch("harness.runner.run_case", side_effect=slow_first), patch("sys.stdout", io.StringIO()) as out:
                run = create_run(config, root / "cases.json", "release")
            order = [json.loads(line)["case"] for line in (run / "results.jsonl").read_text().splitlines()]
            self.assertEqual(order, ["c0", "c1", "c2", "c3"])
            self.assertIn("[4/4] c0", out.getvalue())


if __name__ == "__main__":
    unittest.main()
