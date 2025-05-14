/// # Sample
/// OpenQASM Grover's Search Algorithm
///
/// # Description
/// Grover's search algorithm is a quantum algorithm that finds with high
/// probability the unique input to a black box function that produces a
/// particular output value.
///
/// This program implements the Grover's algorithm for one specific function.

OPENQASM 3;
include "stdgates.inc";

// Define the number of qubits. It must be 5 for this example.
const int nQubits = 5;
// Optimal number of iterations for 5 qubits
int iterations = 4;

/// # Summary
/// Given a register in the all-zeros state, prepares a uniform
/// superposition over all basis states.
def PrepareUniform(qubit[nQubits] q) {
    for int i in [0:nQubits-1] {
        h q[i];
    }
}

/// # Summary
/// Reflects about the basis state marked by alternating zeros and ones.
/// This operation defines what input we are trying to find in the search.
def ReflectAboutMarked(qubit[nQubits] q, qubit aux) {
    // We initialize the outputQubit to (|0⟩ - |1⟩) / √2, so that
    // toggling it results in a (-1) phase.
    x aux;
    h aux;
    // Flip the outputQubit for marked states.
    // Here, we get the state with alternating 0s and 1s by using the X
    // operation on every other qubit.
    for int i in [0:2:nQubits-1] {
        x q[i];
    }
    // Controlled-X operation
    ctrl(nQubits) @ x q[0], q[1], q[2], q[3], q[4], aux;

    // Undo the flips
    for int i in [0:2:nQubits-1] {
        x q[i];
    }
    h aux;
    x aux;
}

// Function to reflect about the uniform superposition
def ReflectAboutUniform(qubit[nQubits] q) {
    // Transform uniform superposition to all-zero
    PrepareUniform(q);

    // Transform all-zero to all-ones
    for int i in [0:nQubits-1] {
        x q[i];
    }

    // Reflect about all-ones
    ctrl(nQubits-1) @ z q[0], q[1], q[2], q[3], q[4];

    // Undo transformations
    for int i in [0:nQubits-1] {
        x q[i];
    }

    // Transform all-zero back to uniform superposition
    PrepareUniform(q);
}

// Main program
qubit[nQubits] q;
qubit aux;
// The state we are looking for is returned after execution.
output bit[nQubits] results;

// Prepare uniform superposition
PrepareUniform(q);

for int i in [1:iterations] {
    ReflectAboutMarked(q, aux);
    ReflectAboutUniform(q);
}

// Measure the qubits
for int i in [0:nQubits-1] {
    results[i] = measure q[i];
}
