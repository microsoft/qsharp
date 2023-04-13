from qsharp import ( interpret_with_dumps, QSharpException )

from IPython.core.magic import (register_cell_magic)


def register_magic():
    # Register the magic on module import. 
    @register_cell_magic
    def qsharp(line, cell):
        "interpret q# code"
        try:
            (value, out, dumps) = interpret_with_dumps(cell)
            return DisplayableOutput(dumps, value)
            # return value
        except QSharpException as ex:
            for diagnostic in ex.diagnostics:
                print("\x1b[31m" + diagnostic.message + "\x1b[0m")


class DisplayableOutput:
    def __init__(self, dumps, value):
        self.dumps = dumps
        self.value = value

    def _repr_html_(self):
        val = ""
        for dump in self.dumps:
            val += self.dump_to_html(dump)

        val += "<p>"
        val += self.value.__repr__()
        val += "</p>"
        return val

    def dump_to_html(self, dump):
        table = '<table>\n'
        for id, r, i, c in dump:
            row = f'<tr><td>|{id}⟩</td><td>{r}</td><td>{i}</td></tr>\n'
            table += row
        table += '</table>'
        return table
