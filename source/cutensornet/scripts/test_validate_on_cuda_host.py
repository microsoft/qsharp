#!/usr/bin/env python3
"""Host-only tests of validate-on-cuda-host.sh's qualification selection.

The tests put test doubles of `uname`, `cargo`, `ldconfig` and the toolchain
commands on `PATH`, so the script believes it runs on a CUDA host. They check
which suite the script selects and how it reports failure; they are not GPU
evidence. Run with
`python3 -m unittest source/cutensornet/scripts/test_validate_on_cuda_host.py`.
When the script's options or the qualification module path change, update
`QUALIFICATION` and the cases below.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("validate-on-cuda-host.sh")
QUALIFICATION = "simulation::mps_execution::qualification::"

COMMAND = r"""
import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
args = sys.argv[1:]
if name == "uname":
    print({"-s": "Linux", "-m": "x86_64"}.get(args[0], "Linux host-only-fixture"))
elif name == "ldconfig":
    print("libcutensornet.so.2 (libc6,x86-64) => /host-only-fixture/libcutensornet.so.2")
    print("libcudart.so.12 (libc6,x86-64) => /host-only-fixture/libcudart.so.12")
elif name == "cargo":
    with open(os.environ["VALIDATOR_COMMAND_LOG"], "a") as log:
        log.write(json.dumps(args) + "\n")
    if args[0] == "test":
        selector = args[args.index("--lib") + 1] if "--lib" in args else "library::tests::fixture"
        mode = os.environ["VALIDATOR_TEST_MODE"]
        if mode == "availability-failure" and "--test" in args:
            print("injected availability command failure")
            sys.exit(1)
        if "::qualification::" in selector:
            if mode == "failure":
                print(f"test {selector} ... FAILED")
                print("test result: FAILED. 0 passed; 1 failed; 0 ignored;")
                sys.exit(1)
            if mode == "zero-tests":
                print("test result: ok. 0 passed; 0 failed; 0 ignored;")
                sys.exit(0)
            if mode == "wrong-selector":
                selector = "unrelated::test"
        print(f"test {selector} ... ok")
        print("test result: ok. 1 passed; 0 failed; 0 ignored;")
    else:
        print("host-only command fixture")
else:
    print("host-only version fixture")
"""


class ValidatorTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="qdk-validator-test-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.log = self.root / "commands.jsonl"
        for name in ("uname", "cargo", "rustc", "rustfmt", "python3", "ldconfig"):
            path = self.root / name
            path.write_text(f"#!{sys.executable}\n" + COMMAND)
            path.chmod(0o700)
        self.env = dict(os.environ)
        self.env.update({
            "PATH": str(self.root) + os.pathsep + self.env.get("PATH", ""),
            "VALIDATOR_COMMAND_LOG": str(self.log),
            "VALIDATOR_TEST_MODE": "success",
        })

    def run_validator(self, *args, mode="success"):
        self.env["VALIDATOR_TEST_MODE"] = mode
        return subprocess.run(
            ["bash", str(SCRIPT), *args],
            env=self.env, capture_output=True, text=True, check=False, timeout=30,
        )

    def numerical_selectors(self):
        if not self.log.exists():
            return []
        commands = [json.loads(line) for line in self.log.read_text().splitlines()]
        selected = []
        for args in commands:
            if args[0] == "test" and "--lib" in args:
                selector = args[args.index("--lib") + 1]
                if "::qualification::" in selector:
                    self.assertEqual(args[args.index("--"):], ["--", "--ignored", "--nocapture"])
                    selected.append(selector)
        return selected

    def test_qualification_runs_the_mps_suite_by_module_path(self):
        result = self.run_validator("--qualification")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.numerical_selectors(), [QUALIFICATION])
        self.assertIn("OK: qualification suite passed", result.stdout)

    def test_qualification_is_not_part_of_default_run(self):
        result = self.run_validator()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.numerical_selectors(), [])
        self.assertIn("SKIPPED (pass --qualification)", result.stdout)

    def test_failure_zero_tests_and_wrong_selector_fail_the_run(self):
        expected = {
            "failure": "FAIL: A100 qualification suite failed",
            "zero-tests": "FAIL: no A100 qualification cases ran",
            "wrong-selector": "FAIL: no A100 qualification cases ran",
        }
        for mode, message in expected.items():
            with self.subTest(mode=mode):
                if self.log.exists():
                    self.log.unlink()
                result = self.run_validator("--qualification", mode=mode)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.numerical_selectors(), [QUALIFICATION])
                self.assertIn(message, result.stdout)

    def test_skip_hardware_combination_is_rejected_before_commands(self):
        result = self.run_validator("--skip-hardware", "--qualification")
        self.assertEqual(result.returncode, 2)
        self.assertIn("cannot be combined", result.stderr)
        self.assertFalse(self.log.exists())


if __name__ == "__main__":
    unittest.main()
