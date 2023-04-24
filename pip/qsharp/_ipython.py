# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from ._qsharp import (_interpret_with_outputs, QSharpException)

from IPython.core.magic import (register_cell_magic)


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
