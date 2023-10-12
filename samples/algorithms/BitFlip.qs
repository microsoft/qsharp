/// # Sample
/// Bit-Flip
///
/// # Description
/// This sample demonstrates the three-qubit bit-flip code. This code is a
/// simple quantum error correction strategy for protecting against single
/// bit-flip error by encoding a logical qubit into three physical qubits. A
/// single bit-flip error is when one of the three physical qubits has its
/// state changed erroneously in a way that is equivalent to applying the X
/// gate to it.
///
/// The bit-flip correction code works by checking the parity of the physical
/// qubits with joint-measurements to preserve the quantum superposition of
/// the qubits. Because all the physical qubits are supposed to have the same
/// state, when the parity checks detect a difference in state, the erroneously
/// flipped qubit can be identified and corrected.
///
/// This Q# program prepares a logical qubit encoded as three physical qubits
/// with one of the qubits being flipped. It then identifies and corrects the
/// flipped qubit.
namespace Sample {
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    @EntryPoint()
    operation Main() : Result[] {
        use physicalQubits = Qubit[3];

        // Prepare a logical qubit with a superposition state from the physical qubits.
        PrepareLocicalQubit(physicalQubits);

        // Induce a bit-flip error on a random qubit.
        X(physicalQubits[DrawRandomInt(0, 2)]);

        // Show the logical qubit with the error state.
        DumpMachine();

        // Find and correct the bit-flip error.
        CorrectError(physicalQubits);

        // Show the logical qubit with the corrected state.
        DumpMachine();

        // Measure the and reset qubits before releasing them.
        let results = MeasureEachZ(physicalQubits);
        ResetAll(physicalQubits);
        return results;
    }

    /// # Summary
    /// This operation prepares a logical qubit in superposition from three physical
    /// qubits by entangling all three physical qubits together, such that, when
    /// measured, all three physical qubits will agree.
    ///
    /// # Input
    /// ## physicalQubits
    /// The given register of three physical qubits to create the logical qubit from.
    /// It is assumed that these qubits are in their default |0〉 state.
    ///
    /// # Output
    /// The operation returns `Unit`. Additionally, the given register of qubits will
    /// be in the state (α|000〉 + β|111〉) / √2, representing a logical qubit in the state
    /// (α|0〉 + β|1〉) / √2.
    operation PrepareLocicalQubit(physicalQubits : Qubit[]) : Unit {
        let alpha = 0.25;
        Ry(2.0 * alpha, Head(physicalQubits));
        ApplyCNOTChain(physicalQubits);
    }

    /// # Summary
    /// This operation detects and corrects a single bit-flip error for a logical
    /// qubit encoded as three physical qubits.
    ///
    /// # Input
    /// ## physicalQubits
    /// The given register of three physical qubits representing a single logical qubit
    /// having superposition (α|0〉 + β|1〉) / √2.
    /// This logical qubit can have up to one bit-flip error that will be corrected.
    ///
    /// # Output
    /// The operation returns `Unit`. Additionally, the given register of qubits will
    /// be in the state: (α|000〉 + β|111〉) / √2.
    operation CorrectError(physicalQubits : Qubit[]) : Unit {

        // Check the parity pair-wise against the Pauli Z basis.
        let parity01 = Measure([PauliZ, PauliZ], physicalQubits[0..1]);
        let parity12 = Measure([PauliZ, PauliZ], physicalQubits[1..2]);
        let parity = (parity01, parity12);

        // Determine which of the three qubits is flipped based on the parity measurements.
        let indexOfError =
            if parity == (One, Zero) {
                0
            } elif parity == (One, One) {
                1
            } elif parity == (Zero, One) {
                2
            } else {
                -1
            };

        // If a qubit was found to be flipped, correct that qubit.
        if indexOfError > -1 {
            X(physicalQubits[indexOfError]);
        }
    }
}
