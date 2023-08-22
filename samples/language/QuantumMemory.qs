/// # Sample
/// Quantum Memory
///
/// # Description
/// The primary quantum feature of Q# is its representation of qubits and qubit
/// memory. Q# supports allocation of qubits, and differentiates between allocating
/// "clean" qubits and "dirty" qubits with the `use` and `borrow` keywords. 
/// Clean qubits are unentangled, whereas dirty qubits are in an unknown state
/// and can potentially be entangled.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Unit {
        // Clean qubits are allocated with `use` statements.
        // Clean qubits are guaranteed to be in a |0⟩ state upon allocation.
        use qubit = Qubit();
        use threeQubits = Qubit[3];

        // Dirty qubits are borrowed with `borrow` statements. Borrowing grants
        // access to qubits that are already allocated but not in use at the time.
        // These qubits are in an arbitrary state, and must be in that same
        // arbitrary state when the borrow statement terminates.
        // Borrowing is useful when reducing the quantum memory requirements of
        // an algorithm.
        borrow qubit = Qubit();
        borrow threeQubits = Qubit[3];
    }
}