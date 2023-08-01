/// # Sample
/// Quantum Teleportation
///
/// # Description
/// Quantum teleportation provides a way of moving a quantum state from one
/// location to another without having to move physical particle(s) along with
/// it. This is done with the help of previously shared quantum entanglement
/// between the sending and the receiving locations, and classical
/// communication.
///
/// This following Q# program implements quantum teleportation.
namespace Microsoft.Quantum.Samples.Teleportation {
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Random; 

    /// # Summary
    /// Sends the state of one qubit to a target qubit by using teleportation.
    ///
    /// Notice that after calling Teleport, the state of `message` is collapsed.
    ///
    /// # Input
    /// ## message
    /// A qubit whose state we wish to send.
    /// ## target
    /// A qubit initially in the |0〉 state that we want to send
    /// the state of message to.
    operation Teleport (message : Qubit, target : Qubit) : Unit {
        // Allocate an auxiliary qubit.
        use auxiliary = Qubit();

        // Create some entanglement that we can use to send our message.
        H(auxiliary);
        CNOT(auxiliary, target);

        // Encode the message into the entangled pair.
        CNOT(message, auxiliary);
        H(message);

        // Measure the qubits to extract the classical data we need to decode
        // the message by applying the corrections on the target qubit
        // accordingly.
        if M(message) == One {
            Z(target);
        }
        if M(auxiliary) == One {
            X(target);
        }

        // Reset auxiliary qubit before releasing.
        Reset(auxiliary);
    }

    /// # Summary
    /// Sets the qubit's state to |+⟩.
    operation SetToPlus(q: Qubit) : Unit {
        Reset(q);
        H(q);
    }

    /// # Summary
    /// Sets the qubit's state to |−⟩.
    operation SetToMinus(q: Qubit) : Unit {
        Reset(q);
        X(q);
        H(q);
    }

    @EntryPoint()
    operation Main () : Unit {
        // Allocate the message and target qubits.
        use (message, target) = (Qubit(), Qubit());

        // Use the `Teleport` operation to send different quantum states.
        // Calls to the `DumpMachine` function are used to show the quantum
        // state before and after teleportation.

        // Teleport the |0〉 state.
        Message("Sending state |0〉");
        DumpMachine();
        Teleport(message, target);
        Message("Received state |0〉");
        DumpMachine();
        ResetAll([message, target]);

        // Teleport the |1〉 state.
        X(message);
        Message("Sending state |1〉");
        DumpMachine();
        Teleport(message, target);
        Message("Received state |1〉");
        DumpMachine();
        ResetAll([message, target]);

        // Teleport the |+〉 state.
        SetToPlus(message);
        Message("Sending state |+〉");
        DumpMachine();
        Teleport(message, target);
        Message("Received state |+〉");
        DumpMachine();
        ResetAll([message, target]);

        // Teleport the |-〉 state.
        SetToMinus(message);
        Message("Sending state |-〉");
        DumpMachine();
        Teleport(message, target);
        Message("Received state |-〉");
        DumpMachine();
        ResetAll([message, target]);
    }
}
