# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (_interpret_with_outputs, QSharpException)
import pathlib

from IPython.core.magic import (register_cell_magic)
from IPython.display import display, Javascript


def register_magic():
    @register_cell_magic
    def qsharp(line, cell):
        "interpret q# code"
        try:
            (value, out) = _interpret_with_outputs(cell)
            return DisplayableOutput(out, value)
        except QSharpException as ex:
            for diagnostic in ex.diagnostics:
                print("\x1b[31m" + diagnostic.message + "\x1b[0m")


class DisplayableOutput:
    def __init__(self, outputs, value):
        self.outputs = outputs
        self.value = value

    def _repr_html_(self):
        val = ""
        for output in self.outputs:
            val += output._repr_html_()

        val += "<p>"
        val += self.value.__repr__()
        val += "</p>"
        return val


def enable_classic_notebook_codemirror_mode():
    """
    Registers %%qsharp cells with MIME type text/x-qsharp
    and defines a CodeMirror mode to enable syntax highlighting.
    This only works in "classic" Jupyter notebooks, not Notebook v7.
    """
    js_to_inject = open(pathlib.Path(__file__).parent.resolve().joinpath(
        ".data", "qsharp_codemirror.js"), mode="r", encoding="utf-8").read()

    # Extend the JavaScript display helper to print nothing when used
    # in a non-browser context (i.e. IPython console)
    class JavaScriptWithPlainTextFallback(Javascript):
        def __repr__(self):
            return ""

    # This will run the JavaScript in the context of the frontend.
    display(JavaScriptWithPlainTextFallback(js_to_inject))
