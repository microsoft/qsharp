/// # Summary
/// OpenQASM Quantum Random Number Generator
///
/// # Description
/// This program implements a quantum random number generator by setting qubits
/// in superposition and then using the measurement results as random bits.

OPENQASM 3;
include "stdgates.inc";

/// # Summary
/// Generates one random bit using a qubit `q`.
def GenerateRandomBit(qubit q) -> bit {
  // Resets qubit `q` to |0〉 state
  reset q;
  // Sets the qubit into superposition of 0 and 1 using the Hadamard gate.
  h q;
  
  // At this point qubit `q` has 50% chance of being measured in the |0〉 state
  // and 50% chance of being measured in the |1〉 state.
  bit b = measure q;

  // Return the measurement result - a random bit.
  return b;
}

/// # Summary
/// Generates a random integer with `nBit` bits using qubit `q`
def GenerateRandomNumber(qubit q, int nBits) -> int {
  int number = 0;

  // Loop `nBits` times to generate `nBits` random bits.
  for int k in [1:nBits] {
      // Shift the number left by 1 to make space for the next bit.
      number <<= 1;
      // Set the least significant bit of the number to a random bit.
      number |= GenerateRandomBit(q);
  }

  // Return the random number.
  return number;
}

// User one qubit `q`.
qubit q;

// Generate a 5-bit random number using the qubit `q`.
int random = GenerateRandomNumber(q, 5);
