import pyqir
from circuit_builder import make_circuit, circuit_to_dict
import json
import argparse


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate circuit JSON from a .ll IR file using pyqir"
    )
    parser.add_argument("llfile", help="path to the .ll IR file")
    parser.add_argument(
        "--indent",
        type=int,
        default=2,
        help="number of spaces to indent JSON output (0 for compact)",
    )
    args = parser.parse_args()

    with open(args.llfile, "r", encoding="utf-8") as f:
        ir_text = f.read()

    context = pyqir.Context()
    module = pyqir.Module.from_ir(ir=ir_text, context=context)

    circuit = make_circuit(module)
    indent = None if args.indent == 0 else args.indent
    print(json.dumps(circuit_to_dict(circuit), indent=indent, ensure_ascii=False))


if __name__ == "__main__":
    main()
