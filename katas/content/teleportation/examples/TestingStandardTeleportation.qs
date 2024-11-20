namespace Kata {
    import Std.Diagnostics.*;

    @EntryPoint()
    operation TestTeleportation() : Unit {

        // To define the different states, let us make use of PauliX, PauliY and PauliZ basis
        let messages = [
            (PauliX, Zero, "|+⟩"),
            (PauliX, One, "|-⟩"),
            (PauliY, Zero, "|i⟩"),
            (PauliY, One, "|-i⟩"),
            (PauliZ, Zero, "|0⟩"),
            (PauliZ, One, "|1⟩")
        ];

        // To effectively test the solution, experiment needs to be repeated multiple times
        let numRepetitions = 100;

        // Loop through all the states to test each one of them individually
        for (basis, sentState, stateName) in messages {

            // Loop through multiple iterations for each state
            for j in 1..numRepetitions {
                // 1. Initialize qubits for Alice and Bob
                // ..

                // 2. Prepare the entangled state between Alice and Bob
                // ..

                // 3. Prepare the Message qubit and send classical message
                // ..

                // 4. Reconstruct Bob's qubit using the classical message
                // ..

                // 5. Verify if the state was teleported correctly. If not, indicate failure
                // ..

                // 6. Reset the qubits
                // ..
            }
        }

        // 7. Indicate success if everything went well
        // ..
    }

    // You might find these helper operations from earlier tasks useful.
    operation PrepareAndSendMessage(qAlice : Qubit, basis : Pauli, state : Result) : (Bool, Bool) {
        use qMessage = Qubit();
        if state == One {
            X(qMessage);
        }
        if basis != PauliZ {
            H(qMessage);
        }
        if basis == PauliY {
            S(qMessage);
        }
        let classicalBits = SendMessage(qAlice, qMessage);
        Reset(qMessage);
        return classicalBits;
    }

    operation ReconstructAndMeasureMessage(qBob : Qubit, (b1 : Bool, b2 : Bool), basis : Pauli) : Result {
        ReconstructMessage(qBob, (b1, b2));
        return Measure([basis], [qBob]);
    }

    operation Entangle(qAlice : Qubit, qBob : Qubit) : Unit is Adj + Ctl {
        H(qAlice);
        CNOT(qAlice, qBob);
    }

    operation SendMessage(qAlice : Qubit, qMessage : Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }

    operation ReconstructMessage(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }
}
