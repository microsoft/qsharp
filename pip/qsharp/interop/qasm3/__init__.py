# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import json
from typing import Any, Dict, List, Optional, Union
from ...estimator import (
    EstimatorResult,
    EstimatorParams,
)
from ..._fs import read_file, list_directory, resolve
from ..._http import fetch_github
from ..._native import resource_estimate_qasm3, Output
from ..._native import run_qasm3 as run_qasm3_native
from ..._qsharp import get_interpreter

# from ..._native import generate_qsharp_from_qasm3


def run(qasm: str, **kwargs) -> Any:
    def callback(output: Output) -> None:
        print(output)

    if "search_path" not in kwargs:
        kwargs["search_path"] = "."

    return run_qasm3_native(
        qasm, callback, read_file, list_directory, resolve, fetch_github, **kwargs
    )


def eval(
    source: str,
    **kwargs,
) -> Any:
    """
    Evaluates OpenQASM 3 source code.
    Output is printed to console.
    :param source: The OpenQASM source code to evaluate.
    :returns value: The value returned by the last statement in the source code.
    :raises QSharpError: If there is an error evaluating the source code.
    :raises QasmError: If there is an error parsing the source code.
    """

    if "search_path" not in kwargs:
        kwargs["search_path"] = "."

    def callback(output: Output) -> None:
        print(output)

    return get_interpreter().interpret_qasm3(source, callback, **kwargs)


def import_callable(
    name: str,
    input: str,
    **kwargs,
) -> Any:
    """
    Evaluates OpenQASM 3 source code.
    Output is printed to console.
    :param name: The name of the callable to generate
    :param input: The OpenQASM source code to evaluate.
    :returns value: The value returned by the last statement in the source code.
    :raises QSharpError: If there is an error evaluating the source code.
    :raises QasmError: If there is an error parsing the source code.
    """

    if "search_path" not in kwargs:
        kwargs["search_path"] = "."

    def callback(output: Output) -> None:
        print(output)

    return get_interpreter().import_qasm3(name, input, callback, **kwargs)


def estimate(
    input: str,
    params: Optional[EstimatorParams] = None,
    **kwargs,
) -> EstimatorResult:
    """
    Performs resource estimation on the supplied OpenQASM 3 program. Gates defined by stdgates.inc will be overridden with definitions from the Q# compiler which allows for more precise estimation compared to definitions based on ```U``` gate decomposition.
    Parameters:
    input (str): The input OpenQASM 3.0 string to be processed.
    use_qasm3_stdgates: Use the qasm3 stdgates.inc definitions for gate implementations instead of the Q# definitions. Defaults to None (False).
    Returns:
    Estimation results (EstimationResult): The detailed EstimationResult generated from processing the supplied input.
    """

    if "search_path" not in kwargs:
        kwargs["search_path"] = "."

    if params is None:
        params = [{}]
    elif isinstance(params, EstimatorParams):
        if params.has_items:
            params = params.as_dict()["items"]
        else:
            params = [params.as_dict()]
    elif isinstance(params, dict):
        params = [params]
    param_str = json.dumps(params)
    res_str = resource_estimate_qasm3(
        input,
        param_str,
        read_file,
        list_directory,
        resolve,
        fetch_github,
        **kwargs,
    )
    res = json.loads(res_str)
    return EstimatorResult(res)


# def convert_qasm3_to_qsharp(
#     input: str,
#     search_path: Optional[List[str]] = None,
# ) -> str:
#     """
#     Converts input qasm3 program to Q# code.
#     Parameters:
#     input (str): The input OpenQASM 3.0 string to be processed.
#     entry_point (str): The entry point for processing. Defaults to 'Main'.
#     search_path (List[str]): List of paths to search in for qasm3 imports. Defaults to an empty list.
#     Returns:
#     Q# code (str): The Q# code generated from the input data.
#     """

#     return generate_qsharp_from_qasm3(input, search_path, read_file, list_directory)
