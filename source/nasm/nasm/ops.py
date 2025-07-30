import re


class Op:
    # type will be 'qir' or 'rt'
    # name will be the operation name, such as 'sx', 'rz', 'cx', 'mz', 'result_record_output', etc.
    # args will be a list of arguments, such as qubit or result ids (int), rangles (float), etc.
    def __init__(self, type: str, name: str, args: list):
        self.type = type
        self.name = name
        self.args = args


class QirOps:
    def __init__(self, ir_text: str):
        self.ir = ir_text

        # We expect attribute 0 to have the required attributes, and be formatted something like:
        #    attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="2" }

        attribute_0 = re.search(
            r'^\s*attributes #0 = \{.*"qir_profiles"="(.+?)".*"required_num_qubits"="(\d+)".*"required_num_results"="(\d+)".*\}\s*$',
            ir_text,
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
            ir_text,
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
