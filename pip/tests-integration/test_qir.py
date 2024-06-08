# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os

from qsharp._native import (
    Interpreter,
    TargetProfile,
    StateDumpData,
    QSharpError,
    Output,
    Circuit,
)
import qsharp
from pyqir import (
    Call,
    Context,
    Module,
    Opcode,
    qubit_id,
    result_id,
    required_num_qubits,
    required_num_results,
)
from typing import Any, Callable, Dict, Optional, Tuple, TypedDict, Union, List
from qirrunner import run, Output, OutputHandler

from qsharp._fs import read_file, list_directory


def get_interpreter(
    target_profile: TargetProfile = TargetProfile.Unrestricted,
    target_name: Optional[str] = None,
) -> Interpreter:
    if isinstance(target_name, str):
        target = target_name.split(".")[0].lower()
        if target == "ionq" or target == "rigetti":
            target_profile = TargetProfile.Base
        elif target == "quantinuum":
            target_profile = TargetProfile.Adaptive_RI
        else:
            raise QSharpError(
                f'target_name "{target_name}" not recognized. Please set target_profile directly.'
            )

    manifest_descriptor = None
    language_features = None
    interpreter = Interpreter(
        target_profile,
        language_features,
        manifest_descriptor,
        read_file,
        list_directory,
    )
    return interpreter


def compile_qsharp(
    source: str,
    target_profile: TargetProfile = TargetProfile.Adaptive_RI,
    target_name: Optional[str] = None,
) -> str:
    interpreter = get_interpreter(target_profile, target_name)
    interpreter.interpret(source)
    qir = interpreter.qir("Test.Main()")
    return qir


def test_compile_qir_input_data() -> None:
    file_path = os.path.join(os.path.dirname(__file__), "resources", "ArithmeticOps.qs")
    with open(file_path, "rt", encoding="utf-8") as file:
        source = file.read()
        output = eval_qsharp(source)
        assert False, output


def read_file(file_name: str) -> str:
    import os

    file_path = os.path.join(os.path.dirname(__file__), "resources", file_name)
    with open(file_path, "rt", encoding="utf-8") as file:
        source = file.read()
    return source


def test_compile_qir_input_qir(file_name: str, expected_output: str) -> None:

    source = read_file(file_name)
    qir = compile_qsharp(source)

    import tempfile

    # create a temporary file to store the qir
    with tempfile.TemporaryDirectory() as tempdir:
        # Create a temporary file in the temporary directory
        with tempfile.NamedTemporaryFile(
            dir=tempdir, delete=True, suffix=".ll"
        ) as temp_file:
            # You can write to the file or read from it
            # encode the uf8 string to bytes
            temp_file.write(qir.encode())
            temp_file.flush()

            handler = OutputHandler()
            run(temp_file.name, None, 1, 42, output_fn=handler.handle)
            output = handler.get_output()
            print(output)
            assert output == expected_output


def test_arithmetic_ops() -> None:
    expected = """START
METADATA	entry_point
METADATA	output_labeling_schema
METADATA	qir_profiles	adaptive_profile
METADATA	required_num_qubits	5
METADATA	required_num_results	5
OUTPUT	TUPLE	4
OUTPUT	INT	5
OUTPUT	INT	25
OUTPUT	INT	0
OUTPUT	INT	243
END	0
"""
    test_compile_qir_input_qir("ArithmeticOps.qs", expected)


def test_bernstein_vazirani_nisq() -> None:
    expected = """START
METADATA	entry_point
METADATA	output_labeling_schema
METADATA	qir_profiles	adaptive_profile
METADATA	required_num_qubits	6
METADATA	required_num_results	5
OUTPUT	ARRAY	5
OUTPUT	RESULT	1
OUTPUT	RESULT	0
OUTPUT	RESULT	1
OUTPUT	RESULT	0
OUTPUT	RESULT	1
END	0
"""
    test_compile_qir_input_qir("BernsteinVaziraniNISQ.qs", expected)
