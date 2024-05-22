namespace Kata.Verification {

    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Diagnostics;
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
    operation StatePrep_BellState (q1 : Qubit, q2 : Qubit, state : Int) : Unit {
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
    // Helper operation that runs teleportation using two building blocks
    // specified as first two parameters.
    operation ComposeTeleportation (
        setupPsiOp : (Qubit => Unit is Adj),
        bellPrepOp : ((Qubit, Qubit) => Unit), 
        getDescriptionOp : ((Qubit, Qubit) => (Bool, Bool)), 
        reconstructOp : ((Qubit, (Bool, Bool)) => Unit), 
        qAlice : Qubit, 
        qBob : Qubit, 
        qMessage : Qubit) : Unit {
        
        setupPsiOp(qMessage);

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

    operation ReconstructMessage_Reference (qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }

    operation CheckTeleportationWithFeedback(protocolTeleportation : (((Qubit => Unit is Adj),Qubit,Qubit,Qubit) => Unit)) : Bool {
        let numRepetitions = 100;
        let setupPsiOps = [I, X, H, Ry(42.0, _)];
        let operationName = ["|0>","|1>","|+>","Unequal Superposition"];
        mutable index = 0;
        while index < Length(setupPsiOps) {
            use (qAlice, qBob, qMessage) = (Qubit(),Qubit(),Qubit());
            protocolTeleportation(setupPsiOps[index],qAlice,qBob,qMessage);
            Adjoint (setupPsiOps[index])(qBob);
            if not CheckZero(qBob){
                Message($"Incorrect. {operationName[index]} state was teleported incorrectly.");
                ResetAll([qMessage, qAlice, qBob]);
                return false;
            }
            ResetAll([qMessage, qAlice, qBob]);
            set index += 1;
        }
        Message("Correct.");
        return true;
    }

    operation CheckTeleportationCompleteWithFeedback(protocolTeleportation : ((Qubit,Qubit,Qubit) => Unit)) : Bool {
        let numRepetitions = 100;
        let setupPsiOps = [I, X, H, Ry(42.0, _)];
        let operationName = ["|0>","|1>","|+>","Unequal Superposition"];
        mutable index = 0;
        while index < Length(setupPsiOps) {
            use (qAlice, qBob, qMessage) = (Qubit(),Qubit(),Qubit());
            (setupPsiOps[index]) (qMessage);
            protocolTeleportation(qAlice,qBob,qMessage);
            Adjoint (setupPsiOps[index])(qBob);
            if not CheckZero(qBob){
                Message($"Incorrect. {operationName[index]} state was teleported incorrectly.");
                ResetAll([qMessage, qAlice, qBob]);
                return false;
            }
            ResetAll([qMessage, qAlice, qBob]);
            set index += 1;
        }
        Message("Correct.");
        return true;
    }

}