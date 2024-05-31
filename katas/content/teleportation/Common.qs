namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    operation EntangleWrapper_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        let (qAlice, qBob) = (qs[0], qs[1]);
        H(qAlice);
        CNOT(qAlice, qBob);
    }

    // ------------------------------------------------------
    // Helper which prepares proper Bell state on two qubits
    // 0 - |Φ⁺⟩ = (|00⟩ + |11⟩) / sqrt(2)
    // 1 - |Φ⁻⟩ = (|00⟩ - |11⟩) / sqrt(2)
    // 2 - |Ψ⁺⟩ = (|01⟩ + |10⟩) / sqrt(2)
    // 3 - |Ψ⁻⟩ = (|01⟩ - |10⟩) / sqrt(2)
    operation StatePrep_BellState(q1 : Qubit, q2 : Qubit, state : Int) : Unit {
        H(q1);
        CNOT(q1, q2);
        
        // now we have |00⟩ + |11⟩ - modify it based on state arg
        if state % 2 == 1 {
            // negative phase
            Z(q2);
        }
        
        if state / 2 == 1 {
            X(q2);
        }
    }

    // ------------------------------------------------------
    // Helper operation that run teleportation using the given operations to prepare the message qubit
    // and the entangled pair, and to run sender and receiver parts of the protocol.
    operation ComposeTeleportation(
        bellPrepOp : ((Qubit, Qubit) => Unit), 
        getDescriptionOp : ((Qubit, Qubit) => (Bool, Bool)), 
        reconstructOp : ((Qubit, (Bool, Bool)) => Unit),
        qAlice : Qubit, 
        qBob : Qubit, 
        qMessage : Qubit) : Unit {

        bellPrepOp(qAlice, qBob);
        let classicalBits = getDescriptionOp(qAlice, qMessage);
        
        // Alice sends the classical bits to Bob.
        // Bob uses these bits to transform his part of the entangled pair into the message.
        reconstructOp(qBob, classicalBits);
    }
    
    operation SendMessage_Reference(qAlice: Qubit, qMessage: Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }

    operation ReconstructMessage_Reference(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }

    // ------------------------------------------------------
    // Helper operation that runs a teleportation operation (specified by teleportOp).
    // The state to teleport is set up using an operation (specified by setupPsiOp).
    //
    // Specifying the state to teleport through an operation allows to get the inverse
    // which makes testing easier.
    operation TeleportTestHelper(
        teleportOp : ((Qubit, Qubit, Qubit) => Unit), 
        setupPsiOp : (Qubit => Unit is Adj),
        psiName: String) : Bool {
        
        use (qMessage, qAlice, qBob) = (Qubit(), Qubit(), Qubit());
        setupPsiOp(qMessage);
            
        // This should modify qBob to be identical to the state
        // of qMessage before the function call.
        teleportOp(qAlice, qBob, qMessage);
            
        // Applying the inverse of the setup operation to qBob
        // should make it Zero.
        Adjoint setupPsiOp(qBob);
        if not CheckZero(qBob) {
            Message($"Incorrect. The state {psiName} was teleported incorrectly.");
            setupPsiOp(qBob);
            Message("The state of the qubits [qMessage, qAlice, qBob] after teleportation:");
            DumpMachine();
            ResetAll([qMessage, qAlice, qBob]);
            return false;
        }
        ResetAll([qMessage, qAlice, qBob]);
        return true;
    }

    // ------------------------------------------------------
    // Run teleportation for a number of different states.
    // After each teleportation success is asserted.
    // Also repeats for each state several times as
    // code is expected to take different paths each time because
    // measurements done by Alice are not deterministic.
    operation TeleportTestLoop(teleportOp : (
        (Qubit, Qubit, Qubit) => Unit)
        ) : Bool {
        // Define setup operations for the message qubit
        // on which to test teleportation: |0⟩, |1⟩, |0⟩ + |1⟩, unequal superposition.
        let setupPsiOps = [(I, "|0⟩"), (X, "|1⟩"), (H, "|+⟩"), (Ry(ArcCos(0.6) * 2.0, _), "0.6|0⟩ + 0.8|1⟩")];
        
        // As part of teleportation Alice runs some measurements
        // with nondeterministic outcome.
        // Depending on the outcomes different paths are taken on Bob's side.
        // We repeat each test run several times to ensure that all paths are checked.
        let numRepetitions = 100;
        for (psiOp, psiName) in setupPsiOps {
            for j in 1 .. numRepetitions {
                if not TeleportTestHelper(teleportOp, psiOp, psiName) {
                    return false;
                }
            }
        }
        Message("Correct.");
        return true;
    }

    operation PrepareAndSendMessage_Reference(qAlice : Qubit, basis : Pauli, state : Bool) : (Bool, Bool) {
        use qMessage = Qubit();
        if state {
            X(qMessage);
        }
        
        if (basis != PauliZ){
            H(qMessage);
        }

        if (basis == PauliY){
            S(qMessage);
        }

        let classicalBits = SendMessage_Reference(qAlice, qMessage);
        Reset(qMessage);
        return classicalBits;
    }

    operation ReconstructAndMeasureMessage_Reference(qBob : Qubit, (b1 : Bool, b2 : Bool), basis : Pauli) : Bool {
        ReconstructMessage_Reference(qBob, (b1, b2));
        return Measure([basis], [qBob]) == One;
    }

    // ------------------------------------------------------
    // Runs teleportation for each state that is to be prepared and
    // sent by Alice. Success is asserted after each teleportation.
    // Also repeats for each state several times; this is because
    // code is expected to take different paths each time since
    // measurements done by Alice are not deterministic.
    operation TeleportPreparedStateTestLoop(
        prepareAndSendMessageOp : ((Qubit, Pauli, Bool) => (Bool, Bool)), 
        reconstructAndMeasureMessageOp : ((Qubit, (Bool, Bool), Pauli) => Bool)
        ) : Bool {
        
        let messages = [(PauliX, false, "|+>"), 
                        (PauliX, true, "|->"), 
                        (PauliY, false, "|i>"), 
                        (PauliY, true, "|-i>"), 
                        (PauliZ, false, "|0>"), 
                        (PauliZ, true, "|1>")];
        let numRepetitions = 100;
        
        use (qAlice, qBob) = (Qubit(), Qubit());
        for (basis, sentState, stateName) in messages {
            for j in 1 .. numRepetitions {
                StatePrep_BellState(qAlice, qBob, 0);
                let classicalBits = prepareAndSendMessageOp(qAlice, basis, sentState);
                let receivedState = reconstructAndMeasureMessageOp(qBob, classicalBits, basis);
                
                if (sentState != receivedState){
                    Message($"Sent and received states were not equal for {stateName}.");
                    ResetAll([qAlice, qBob]);
                    return false;
                }
                ResetAll([qAlice, qBob]);
            }
        }
        Message($"Correct");
        return true;
    }

    operation TeleportEntanglementSwappingTestLoop(
        entanglementSwapping : ((Qubit, Qubit) => Int, (Qubit, Int) => Unit)
    ) : Bool {
        
        for i in 1 .. 15 {
            use (qAlice1, qAlice2) = (Qubit(), Qubit());
            EntangleWrapper_Reference([qAlice1, qAlice2]);

            use (qBob1, qBob2) = (Qubit(), Qubit());
            EntangleWrapper_Reference([qBob1, qBob2]);
            
            let (teleportOp, adjustOp) = entanglementSwapping;
            // Apply the operations returned by the solution: 
            // first Charlie's side, then Bob's side.
            let result = teleportOp(qAlice1, qBob1);
            adjustOp(qBob2, result);

            // Apply adjoint of the operation that prepares the |Φ⁺⟩ state:
            // if the state of Alice's and Bob's qubits was |Φ⁺⟩,
            // their state should become |00⟩ now.
            Adjoint EntangleWrapper_Reference([qAlice2, qBob2]);
            
            // Assert that Alice's and Bob's qubits end up in |0⟩ state.
            if not CheckAllZero([qAlice2, qBob2]){
                Message($"The entanglement swapping was not successful.");
                ResetAll([qAlice1, qAlice2, qBob1, qBob2]);
                return false;
            }
            
            ResetAll([qAlice1, qAlice2, qBob1, qBob2]);
            return true;
        }
    }

    operation MeasurementFreeTeleportTestLoop(
        measurementfreeteleport : ((Qubit, Qubit, Qubit) => Unit)
    ) : Bool {
        let setupPsiOps = [(I, "|0⟩"), (X, "|1⟩"), (H, "|+⟩"), (Ry(ArcCos(0.6) * 2.0, _), "0.6|0⟩ + 0.8|1⟩")];
        let numRepetitions = 100;
        
        use (qMessage, qAlice, qBob) = (Qubit(), Qubit(), Qubit());
        for (psiOp, psiName) in setupPsiOps {
            for j in 1 .. numRepetitions {
                psiOp(qMessage);
                StatePrep_BellState(qAlice, qBob, 0);
                measurementfreeteleport(qAlice, qBob, qMessage);
                Adjoint psiOp(qBob);
                if not CheckZero(qBob){
                    Message($"Incorrect. The state {psiName} was teleported incorrectly.");
                    psiOp(qBob);
                    Message("The state of the qubits [qMessage, qAlice, qBob] after teleportation:");
                    DumpMachine();
                    ResetAll([qMessage, qAlice, qBob]);
                    return false;
                }        
                ResetAll([qMessage, qAlice, qBob]);
            }
        }

        Message("Correct.");
        return true;
    }

    operation EntangleThreeQubitsWrapper_Reference(qs : Qubit[]) : Unit is Adj {
        let (qAlice, qBob, qCharlie) = (qs[0], qs[1], qs[2]);
        // Starting with |000⟩
        H(qBob); // now state is: 1/sqrt(2) (|000⟩ + |010⟩)
        CNOT(qBob, qCharlie); // state: 1/sqrt(2) (|000⟩ + |011⟩)
        H(qAlice); // state: 1/2 (|000⟩ + |011⟩ + |100⟩ + |111⟩)
        CNOT(qAlice, qCharlie); // final state:  1/2 (|000⟩ + |011⟩ + |101⟩ + |110⟩)
    }

    operation ReconstructMessageWhenThreeEntangledQubitsTestLoop(
        reconstructmessagewhenthreeentangledqubits : ((Qubit, (Bool, Bool), Bool) => Unit)
    ) : Bool {
        
        let setupPsiOps = [(I, "|0⟩"), (X, "|1⟩"), (H, "|+⟩"), (Ry(ArcCos(0.6) * 2.0, _), "0.6|0⟩ + 0.8|1⟩")];
        let numRepetitions = 100;
        
        use (qMessage, qAlice, qBob, qCharlie) = (Qubit(), Qubit(), Qubit(), Qubit());
        for (psiOp, psiName) in setupPsiOps {
            for j in 1 .. numRepetitions {
                psiOp(qMessage);
                EntangleThreeQubitsWrapper_Reference([qAlice, qBob, qCharlie]);
                let (b1, b2) = SendMessage_Reference(qAlice, qMessage);
                let b3 = ResultAsBool(M(qBob));
                reconstructmessagewhenthreeentangledqubits(qCharlie, (b1, b2), b3);
                Adjoint psiOp(qCharlie);
                if not CheckZero(qCharlie){
                    Message($"Incorrect. The state {psiName} was teleported incorrectly.");
                    psiOp(qCharlie);
                    Message("The state of the qubits [qMessage, qAlice, qBob, qCharlie] after teleportation:");
                    DumpMachine();
                    ResetAll([qMessage, qAlice, qBob, qCharlie]);
                    return false;
                }
                ResetAll([qMessage, qAlice, qBob, qCharlie]);
            }
        }
        Message($"Correct.");
        return true;
    }
}