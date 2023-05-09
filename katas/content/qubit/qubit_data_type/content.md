# Qubit data type

In Q#, qubits are represented by the `Qubit` data type. On a physical quantum computer, it's impossible to directly access the state of a qubit, whether to read its exact state, or to set it to a desired state, and this data type reflects that. Instead, you can change the state of a qubit using quantum gates, and extract information about the state of the system using measurements.

That being said, when you run Q# code on a quantum simulator instead of a physical quantum computer, you can use diagnostic functions that allow you to peek at the state of the quantum system. This is very useful both for learning and for debugging small Q# programs.

The qubits aren't an ordinary data type, so the variables of this type have to be declared and initialized ("allocated") a little differently:

Freshly allocated qubits start out in state $|0\rangle$, and have to be returned to that state by the time they are released. If you attempt to release a qubit in any state other than $|0\rangle$ will result in a runtime error. We will see why it is important later, when we look at multi-qubit systems.

## Examining Qubit States in Q#

We will be using the function [`DumpMachine`](https://docs.microsoft.com/qsharp/api/qsharp/microsoft.quantum.diagnostics.dumpmachine) to print the state of the quantum computer.

The exact behavior of this function depends on the quantum simulator or processor you are using.

On the simulator used in this demo, this function prints the information on each basis state that has a non-zero amplitude, one basis state per row.
This includes information about the amplitude of the state, the probability of measuring that state, and the phase of the state (more on that later)

Each row has the following format:

<table class="state-table"><thead><tr><th>Basis State<br>(|𝜓ₙ…𝜓₁⟩)</th><th>Amplitude</th><th>Measurement Probability</th><th colspan="2">Phase</th></tr></thead></table>

For example, the state $|0\rangle$ would be represented as follows:

<table class="state-table"><tbody><tr><td style="text-align: center;">|0⟩</td><td style="text-align: right;">1.0000+0.0000𝑖</td><td style="display: flex; justify-content: space-between; padding: 8px 20px;"><progress max="100" value="100" style="width: 40%;"></progress><span>100.0000%</span></td><td style="transform: rotate(0rad);">↑</td><td style="text-align: right;">0.0000</td></tr></tbody></table>

> It is important to note that although we reason about quantum systems in terms of their state, Q# does not have any representation of the quantum state in the language. Instead, state is an internal property of the quantum system, modified using gates. For more information, see [Q# documentation on quantum states](https://docs.microsoft.com/azure/quantum/concepts-dirac-notation#q-gate-sequences-equivalent-to-quantum-states).

This demo shows how to allocate a qubit and examine its state in Q#. This demo uses quantum gates to manipulate the state of the qubit - we will explain how they work in the next tutorial, so do not worry about them for now. Run the next example to see the output:
