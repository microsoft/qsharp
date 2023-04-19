from qsharp import ( interpret_with_dumps, QSharpException )

# TODO: pylance complains about this import, but it works. Thanks copilot!
from IPython.core.magic import (register_cell_magic)


def register_magic():
    # Register the magic on module import. 
    @register_cell_magic
    def qsharp(line, cell):
        "interpret q# code"
        try:
            (value, out) = interpret_with_dumps(cell)
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