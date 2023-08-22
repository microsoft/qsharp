/// # Sample
/// Diagnostics
///
/// # Description
/// The `Microsoft.Quantum.Diagnostics` namespace provides functionality for inspecting and
/// diagnosing program state. This is useful when debugging or diagnosing behavior.
/// Q# provides a number of features designed to enable testing and debugging Q# programs,
/// primarily in the form of assertions and messaging classical state to the user.
namespace MyQuantumApp {
    open Microsoft.Quantum.Diagnostics;
    @EntryPoint()
    operation Main(): Unit {
        // `Message` emits a debug log.
        Message("Program is starting.");

        use qs = Qubit[2];
        CNOT(qs[0], qs[1]);
        H(qs[0]);

        // We may now want to see the state of `qs` after applying some gates to them.
        // `DumpMachine` displays the quantum state of the qubits.
        DumpMachine();

        // `Fact` checks whether a classical condition is true, and throws an
        // error with the specified message if not.
        Fact(1 == 1, "1 should always be equal to 1");
    }
}