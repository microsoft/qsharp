import json
import re


class Op:
    # type will be 'qir' or 'rt'
    # name will be the operation name, such as 'sx', 'rz', 'cx', 'mz', 'result_record_output', etc.
    # args will be a list of arguments, such as qubit or result ids (int), rangles (float), None for i8* null, etc.
    def __init__(self, type: str, name: str, args: list):
        self.type = type
        self.name = name
        self.args = args

    def __repr__(self):
        return f"{self.name}{self.args}"


class QirOps:
    def __init__(self, ir_text: str):
        self.ir = str(ir_text)

        # We expect attribute 0 to have the required attributes, and be formatted something like:
        #    attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }

        attribute_0 = re.search(
            r'^\s*attributes #0 = \{.*"qir_profiles"="(.+?)".*"required_num_qubits"="(\d+)".*"required_num_results"="(\d+)".*\}\s*$',
            self.ir,
            re.MULTILINE,
        )
        if not attribute_0:
            raise ValueError("QIR does not contain the required attributes")

        self.profile = attribute_0.group(1)
        self.qubits = int(attribute_0.group(2))
        self.results = int(attribute_0.group(3))

        if self.profile != "base_profile":
            raise ValueError(f"Profile is not base_profile: {self.profile}")

        if self.qubits < 0 or self.results < 0:
            raise ValueError(f"Qubit and result count must be greater than 0")

        # Extract the body of the entry point function
        #   First non-capture group is the #0 attribute and <block_name>: label
        #   With no branching, end at the 'ret' instruction
        entry_point_body = re.search(
            r"(?:define void @\S+\(\) #0\s+{\n\w+:\n)((.|\n)*)(?:^\s*ret (i64|void))",
            self.ir,
            re.MULTILINE,
        )
        if not entry_point_body:
            raise ValueError("QIR does not contain entry point function body")

        self.entry_point_body = entry_point_body.group(1)

        self.ops = [
            self.parse_line(line)
            for line in self.entry_point_body.splitlines()
            if line.strip()
        ]

    """
    A program such as

      operation Main() : Result[] {
        use q = Qubit[2];
        SX(q[0]);
        Rz(0.5, q[1]);
        CZ(q[0], q[1]);
        return [M(q[0]), M(q[1])];
      }

    Results in QIR such as:

      call void @__quantum__qis__sx__body(%Qubit* inttoptr (i64 0 to %Qubit*))
      call void @__quantum__qis__rz__body(double 0.5, %Qubit* inttoptr (i64 1 to %Qubit*))
      call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
      call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
      call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
      call void @__quantum__rt__array_record_output(i64 2, i8* null)
      call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
      call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)

    Note: It may (should) also have an initial "call void @__quantum__rt__initialize(i8* null)" call
    """

    def parse_line(self, line: str) -> Op:
        call_groups = re.search(
            r"^\s*call\s+void\s+@__quantum__(qis|rt)__(\w+?)\((.*)\)\s*$", line
        )
        if not call_groups or len(call_groups.groups()) != 3:
            raise ValueError(f"Invalid QIR line: {line}")

        name = call_groups.group(2)
        if name.endswith("__body"):
            name = name[:-6]

        # Normalize 'm' to 'mz' for measurement operations
        if name == "m":
            name = "mz"

        return Op(
            type=call_groups.group(1),
            name=name,
            args=self.parse_args(call_groups.group(3)),
        )

    def parse_args(self, args: str):
        # Input will be something like:
        #    double 0.5, %Qubit* inttoptr (i64 1 to %Qubit*)
        #    %Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*)
        #    %Result* inttoptr (i64 0 to %Result*), i8* null
        results = []
        split_args = [arg.strip() for arg in args.split(",")]
        for arg in split_args:
            if not arg:
                continue
            if arg.startswith("%Qubit* inttoptr (i64 ") or arg.startswith(
                "%Result* inttoptr (i64 "
            ):
                # Extract the qubit or result id from the inttoptr
                match = re.search(r"\(i64 (\d+) to \%", arg)
                if match:
                    results.append(int(match.group(1)))
                else:
                    raise ValueError(f"Invalid QIR argument: {arg}")
            elif arg.startswith("double ") or arg.startswith("i64 "):
                # Extract the number from the double or i64 argument
                match = re.search(r"^(?:double|i64)\s+(.*)$", arg)
                if match:
                    if arg.startswith("double "):
                        results.append(float(match.group(1)))
                    else:
                        results.append(int(match.group(1)))
                else:
                    raise ValueError(f"Invalid QIR argument: {arg}")
            elif arg == "i8* null":
                # Shouldn't get non null i8* arguments in base QIR (i.e. no strings)
                results.append(None)
            else:
                raise ValueError(f"Invalid QIR argument: {arg}")
        return results

    def transpose(self):
        self.ops = transpose(self.ops)

    def json(self):
        # Return a circuit representation of the QIR operations
        result = {"operations": [], "qubits": []}
        for op in self.ops:
            if op.type == "rt":
                continue
            qubits = (
                [op.args[0]]
                if op.name
                in ["sx", "mz", "x", "y", "z", "h", "s", "s_adj", "t", "t_adj"]
                else (
                    op.args[:2]
                    if op.name in ["cz", "cx"]
                    else (op.args[1:] if op.name in ["rz", "rx", "ry", "rzz"] else [])
                )
            )
            entry = {
                "gate": op.name,
                "targets": [{"qId": q, "type": 0} for q in qubits],
            }
            if len(op.args) > 0 and isinstance(op.args[0], float):
                entry["displayArgs"] = "{:.4f}".format(op.args[0])
            # if op.name == "mz":
            #     # entry["gate"] = "Measure"
            #     entry["isMeasurement"] = True
            result["operations"].append(entry)
        for i in range(self.qubits):
            result["qubits"].append({"id": i, "numChildren": 0})

        # Return the JSON string representation of the circuit
        return json.dumps(result)

    def __str__(self):
        return f"QirOps(profile={self.profile}, qubits={self.qubits}, results={self.results}, ops={self.ops})"


def transpose(input: list[Op]) -> list[Op]:
    """
    Transpose the input operations to match the expected sx, rz, cz, and mz instruction set
    """
    pi = 3.141592653589793
    pi_2 = pi / 2
    pi_4 = pi / 4

    skip = ["initialize", "barrier"]
    native = ["sx", "rz", "cz", "mz", "array_record_output", "result_record_output"]
    mappings = {
        "x": lambda args: [Op("qir", "sx", args), Op("qir", "sx", args)],
        "y": lambda args: [
            Op("qir", "sx", args),
            Op("qir", "sx", args),
            Op("qir", "rz", [pi, args[0]]),
        ],
        "z": lambda args: [Op("qir", "rz", [pi, args[0]])],
        "h": lambda args: [
            Op("qir", "rz", [pi_2, args[0]]),
            Op("qir", "sx", args),
            Op("qir", "rz", [pi_2, args[0]]),
        ],
        "s": lambda args: [Op("qir", "rz", [pi_2, args[0]])],
        "s_adj": lambda args: [Op("qir", "rz", [-pi_2, args[0]])],
        "t": lambda args: [Op("qir", "rz", [pi_4, args[0]])],
        "t_adj": lambda args: [Op("qir", "rz", [-pi_4, args[0]])],
        "rx": lambda args: [
            Op("qir", "h", [args[1]]),
            Op("qir", "rz", args),
            Op("qir", "h", [args[1]]),
        ],
        "ry": lambda args: [
            Op("qir", "h", [args[1]]),
            Op("qir", "s", [args[1]]),
            Op("qir", "h", [args[1]]),
            Op("qir", "rz", args),
            Op("qir", "h", [args[1]]),
            Op("qir", "s_adj", [args[1]]),
            Op("qir", "h", [args[1]]),
        ],
        "cx": lambda args: [
            Op("qir", "h", [args[1]]),
            Op("qir", "cz", args),
            Op("qir", "h", [args[1]]),
        ],
        "rzz": lambda args: [
            Op("qir", "h", [args[1]]),
            Op("qir", "cz", args[1:]),
            Op("qir", "h", [args[1]]),
            Op("qir", "rz", [args[0], args[1]]),
            Op("qir", "h", [args[1]]),
            Op("qir", "cz", args[1:]),
            Op("qir", "h", [args[1]]),
        ],
    }
    output = []
    did_mapping = True
    while did_mapping:  # Continue until no mappings are applied
        did_mapping = False
        output = []
        for op in input:
            if op.name in native:
                output.append(op)
            elif op.name in mappings:
                did_mapping = True
                output.extend(mappings[op.name](op.args))
            elif op.name in skip:
                continue
            else:
                raise ValueError(f"Unsupported operation {op.name} in QIR code")
        input = output

    return reduce(output)


def reduce(input: list[Op]) -> list[Op]:
    queued_ops = {}
    output = []

    # NOTE: Could also ignore consequtive cz ops on the same qubits, but seems rare.

    def flush_op(id: int):
        if id in queued_ops:
            queued_op = queued_ops[id]
            # Clamp the angle to [0, 2*pi]
            queued_op.args[0] = queued_op.args[0] % (2 * 3.141592653589793)

            # Skip if Rz(0) (to within 1e-10)
            if abs(queued_op.args[0]) >= 1e-10:
                output.append(queued_op)
            del queued_ops[id]

    for op in input:
        if op.name == "rz":
            # If the operation is a rz, we need to check if we can combine it with a previous rz
            if op.args[1] in queued_ops:
                # Accumulate the angle
                queued_ops[op.args[1]].args[0] += op.args[0]
            else:
                # New rz operation, queue it
                queued_ops[op.args[1]] = op
        elif op.name in ["sx", "mz"]:
            flush_op(op.args[0])
            output.append(op)
        elif op.name == "cz":
            flush_op(op.args[0])
            flush_op(op.args[1])
            output.append(op)
        elif op.name in ["array_record_output", "result_record_output"]:
            output.append(op)
        else:
            raise ValueError(f"Unsupported operation {op.name} in QIR code to reduce")

    # Flush any remaining queued operations (shouldn't be any if all measured at the end)
    for op in queued_ops.values():
        output.append(op)

    return output
