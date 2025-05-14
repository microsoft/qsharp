# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

"""
_ipython.py

This module provides IPython magic functions for integrating Q# code
execution within Jupyter notebooks.
"""

from typing import Any, Callable, Dict, List, Optional, Tuple, Union
from time import monotonic
from IPython.display import display, Javascript, clear_output
from IPython.core.magic import register_cell_magic
from IPython.core.magic_arguments import (
    argument,
    magic_arguments,
    parse_argstring,
    argument_group,
)
from ._native import ProgramType, QSharpError, QasmError  # type: ignore
from ._qsharp import get_interpreter, TargetProfile
from . import telemetry_events
import pathlib


def d(output):
    display(output)
    # This is a workaround to ensure that the output is flushed. This avoids an issue
    # where the output is not displayed until the next output is generated or the cell
    # is finished executing.
    display(display_id=True)


def register_magic():
    @register_cell_magic
    def qsharp(line, cell):
        """Cell magic to interpret Q# code in Jupyter notebooks."""
        # This effectively pings the kernel to ensure it recognizes the cell is running and helps with
        # accureate cell execution timing.
        clear_output()

        def callback(output):
            display(output)
            # This is a workaround to ensure that the output is flushed. This avoids an issue
            # where the output is not displayed until the next output is generated or the cell
            # is finished executing.
            display(display_id=True)

        telemetry_events.on_run_cell()
        start_time = monotonic()

        try:
            results = get_interpreter().interpret(cell, callback)

            durationMs = (monotonic() - start_time) * 1000
            telemetry_events.on_run_cell_end(durationMs)

            return results
        except QSharpError as e:
            # pylint: disable=raise-missing-from
            raise QSharpCellError(str(e))

    @magic_arguments()
    @argument_group("Interactive")
    @argument(
        "--name",
        "-n",
        type=str,
        help=("Create callable with given name"),
    )
    @argument_group("Simulation")
    @argument(
        "--shots",
        "-s",
        type=int,
        default=1,
        help=("Number of shots to run"),
    )
    @argument_group("Compilation")
    @argument(
        "--compile",
        "-c",
        help=("Compile the OpenQASM code to a QIR"),
    )
    @argument(
        "--target_profile",
        "-t",
        type=type(TargetProfile),
        default=TargetProfile.Adaptive_RIF,
        help=("Target profile to use for compilation"),
    )
    @register_cell_magic
    def openqasm(line, cell):
        """Cell magic to interpret OpenQASM code in Jupyter notebooks."""
        # This effectively pings the kernel to ensure it recognizes the cell is running and helps with
        # accureate cell execution timing.
        clear_output()
        telemetry_events.on_run_cell()
        start_time = monotonic()

        try:
            from .qasm import import_qasm, run, compile

            args = parse_argstring(openqasm, line)
            if args.name is None:
                shots = args.shots
                results = run(cell, shots=shots)
            elif args.compile:
                results = compile(cell, target_profile=args.target_profile)
            else:
                results = import_qasm(cell, name=args.name)
            durationMs = (monotonic() - start_time) * 1000
            telemetry_events.on_run_cell_end(durationMs)

            return results
        except QSharpError as e:
            # pylint: disable=raise-missing-from
            raise QSharpCellError(str(e))
        except QasmError as e:
            raise QSharpCellError(str(e))


def enable_classic_notebook_codemirror_mode():
    """
    Registers %%qsharp cells with MIME type text/x-qsharp
    and defines a CodeMirror mode to enable syntax highlighting.
    This only works in "classic" Jupyter notebooks, not Notebook v7.
    """
    js_to_inject = open(
        pathlib.Path(__file__)
        .parent.resolve()
        .joinpath(".data", "qsharp_codemirror.js"),
        mode="r",
        encoding="utf-8",
    ).read()

    # Extend the JavaScript display helper to print nothing when used
    # in a non-browser context (i.e. IPython console)
    class JavaScriptWithPlainTextFallback(Javascript):
        def __repr__(self):
            return ""

    # This will run the JavaScript in the context of the frontend.
    display(JavaScriptWithPlainTextFallback(js_to_inject))


class QSharpCellError(BaseException):
    """
    Error raised when a %%qsharp cell fails.
    """

    def __init__(self, traceback: str):
        self.traceback = traceback.splitlines()

    def _render_traceback_(self):
        # We want to specifically override the traceback so that
        # the Q# error directly from the interpreter is shown
        # instead of the Python error.
        return self.traceback
