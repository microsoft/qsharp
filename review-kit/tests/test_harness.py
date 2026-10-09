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
from harness.builds import verify_build  # noqa: E402
from harness.cases import load_cases, snapshot_project  # noqa: E402
from harness.common import (  # noqa: E402
    KIT, ReviewError, isolated, load_config, read_json, tree_digest, validate, write_json,
)
from harness.discovery import infer_entry, snapshot_issues  # noqa: E402
from harness.investigate import ddmin  # noqa: E402
from harness.records import encode, parse_output, validate_value  # noqa: E402
from harness.report import generate_report, source_repro  # noqa: E402
from harness.runner import compare, endpoint, prune_qir, run_case  # noqa: E402
from harness import worker  # noqa: E402


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


if __name__ == "__main__":
    unittest.main()
