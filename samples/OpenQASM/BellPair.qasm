/// # Summary
/// OpenQASM Bell Pair sample
///
/// # Description
/// Bell pairs are specific quantum states of two qubits that represent
/// the simplest (and maximal) examples of quantum entanglement. This sample
/// prepares |Φ⁺⟩ = (|00⟩+|11⟩)/√2.
///
/// # References
/// - [Bell state](https://en.wikipedia.org/wiki/Bell_state)

OPENQASM 3;
include "stdgates.inc";

// Declares use of qubits `q1` and `q2`.
qubit q1;
qubit q2;

// Set qubit `q1` in superposition of |0⟩ and |1⟩ by applying a Hadamard gate.
h q1;
// Entangle the two qubits `q1` and `q2` using the `cx` gate.
cx q1, q2;

// Measure the two qubits and store results in classical variables `r1` and `r2`.
bit r1 = measure q1;
bit r2 = measure q2;

// Note, that the reported result (r1, r2)
// will always be either (Zero, Zero) or (One, One).
