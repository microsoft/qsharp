/// # Summary
/// OpenQASM Hello World sample
///
/// # Description
/// This is one of the simplest Q# programs that contains quantum part.
/// It uses one qubit, resets it and and immediately measures it.
/// Since the qubit is in |0〉 state after reset
/// such measurement will always yield `Zero`.

// OpenQASM version identifier.
OPENQASM 3;
// This `include` directive allows standard gates to be used.
include "stdgates.inc";

// Declares use of a single qubit and names it `q`.
// All qubits must be declared in a global scope.
qubit q;

// Qubits are initially in an undefined state.
// Reset is used here to initialize qubit to a |0〉 state.
reset q;

// Measures qubit and stores the result in a classical bit.
// The qubit is in |0〉 state after reset, so the `b` will be 0.
bit b = measure q;

// Note, that the content of the classical variables
// are reported to the user at the end of the program.
