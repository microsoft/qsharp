## Qubit data type

In Q#, qubits are represented by the `Qubit` data type. On a physical quantum computer, it's impossible to directly access the state of a qubit, whether to read its exact state, or to set it to a desired state, and this data type reflects that. Instead, you can change the state of a qubit using [quantum gates](../SingleQubitGates/SingleQubitGates.ipynb), and extract information about the state of the system using measurements.

That being said, when you run Q# code on a quantum simulator instead of a physical quantum computer, you can use diagnostic functions that allow you to peek at the state of the quantum system. This is very useful both for learning and for debugging small Q# programs.

The qubits aren't an ordinary data type, so the variables of this type have to be declared and initialized (\"allocated\") a little differently:

Freshly allocated qubits start out in state $|0\\rangle$, and have to be returned to that state by the time they are released. If you attempt to release a qubit in any state other than $|0\\rangle$, your program will throw a `ReleasedQubitsAreNotInZeroStateException`. We will see why it is important later, when we look at multi-qubit systems.
