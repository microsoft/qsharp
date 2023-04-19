import qsharp
import pathlib
from qsharp import (CompilationException, RuntimeException)

sample_qs = pathlib.Path(__file__).parent.resolve().joinpath("./sample.qs")

# Interpret some code.
# Just defining an operation.
# TODO: Q: namespace suport?
qsharp.interpret("""
operation Main() : Unit {
    use q = Qubit();
    let r = M(q);
    Message("Result: " + AsString(r));
}
""")

# Interpret some more code.
# Should output Message output to stdout.
# TODO: Do something fancy for notebooks
qsharp.interpret("Main()")

# Add some source from a file
qsharp.interpret_file(sample_qs)

# Call an operation from the file
qsharp.interpret("AllBasisVectorsWithPhases_TwoQubits()")

# Get result
result = qsharp.interpret("1 + 2")

print(f"Result was: {result} (type: {type(result).__name__})")

# Return a Result type
print(qsharp.interpret("One"))

# Return a bool type
print(qsharp.interpret("true"))

# Return a Pauli type
print(qsharp.interpret("PauliX"))

# Compile time error - parse error
try:
    qsharp.interpret("operation Foo() {}")
except CompilationException as ex:
    for diagnostic in ex.diagnostics:
        print("\x1b[33m" + diagnostic.message + "\x1b[0m")

# Compile time errors - multiple errors
try:
    qsharp.interpret("operation Foo() : Unit { Bar(); Baz(); }")
except CompilationException as ex:
    for diagnostic in ex.diagnostics:
        print("\x1b[33m" + diagnostic.message + "\x1b[0m")

# Runtime error
try:
    qsharp.interpret("let (x, y, z) = (0, 1);")
except RuntimeException as ex:
    for diagnostic in ex.diagnostics:
        print("\x1b[31m" + diagnostic.message + "\x1b[0m")

# State visualization
(value, outputs) = qsharp.interpret_with_dumps(
    "AllBasisVectorsWithPhases_TwoQubits()")
print(f"States: {outputs}")

# Multiple statements
print(qsharp.interpret("3; 4; 5"))

# No statements
print(qsharp.interpret(""))

# Return a tuple
print(qsharp.interpret("(1,2,\"hi\")"))

# Check bit order in state output
qsharp.interpret("""
operation AllocateQubits() : Unit {
    use qs = Qubit[2];
    use q3 = Qubit();
    use q4 = Qubit();
    
    H(qs[0]);
    H(q3);
    X(q4);

    Microsoft.Quantum.Diagnostics.DumpMachine();
}
""")

qsharp.interpret("""
AllocateQubits()
""")
