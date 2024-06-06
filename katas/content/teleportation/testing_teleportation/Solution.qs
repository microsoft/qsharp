namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation TestTeleportation() : Unit {

        // To define the different states, let us make use of PauliX, PauliY and PauliZ basis
        let messages = [(PauliX, Zero, "|+⟩"), 
                        (PauliX, One, "|-⟩"), 
                        (PauliY, Zero, "|i⟩"), 
                        (PauliY, One, "|-i⟩"), 
                        (PauliZ, Zero, "|0⟩"), 
                        (PauliZ, One, "|1⟩")];

        // To effectively test the solution, experiment needs to be repeated multiple times
        let numRepetitions = 100;

        // 1. Initialize qubit for Alice and Bob
        use (qAlice, qBob) = (Qubit(), Qubit());

        // Loop through all the states to test each one of them individually
        for (basis, sentState, stateName) in messages {

            // Loop through multiple iterations for each state
            for j in 1 .. numRepetitions {

                // 2. Prepare the entangled state between Alice and Bob
                Entangle(qAlice, qBob);

                // 3. Prepare the Message qubit and send classical message
                let classicalBits = PrepareAndSendMessage(qAlice, basis, sentState);

                // 4. Reconstruct Bob's qubit using the measurement result
                let receivedState = ReconstructAndMeasureMessage(qBob, classicalBits, basis);

                // 5. Verify if the state was teleported correctly. If not, indicate failure
                if sentState != receivedState {
                    Message($"Received incorrect basis state when sending {stateName} in the {basis} basis.");
                    ResetAll([qAlice, qBob]);
                }

                // 6. Reset the qubits
                ResetAll([qAlice, qBob]);
            }
        }

        // 7. Indicate success if everything went well
        Message($"Correct");
    }

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

    operation SendMessage(qAlice: Qubit, qMessage: Qubit) : (Bool, Bool) {
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