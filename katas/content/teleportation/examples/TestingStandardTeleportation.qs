namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation RunExample() : Unit {

        let psiStates = [(0,"|0⟩"), (1,"|1⟩"), (2,"|+⟩"), (3,"|-⟩"), (4,"|i⟩"), (5,"|-i⟩")];

        for (psiStateNumber, psiState) in psiStates {
            use (qAlice, qBob, qMessage) = (Qubit(), Qubit(), Qubit());
            
            // Preparing state to be teleported
            Message($"Preparing state {psiState} to be teleported.");
            if (psiStateNumber % 2 == 1){
                X(qMessage);
            }
            if (psiStateNumber >= 2){
                H(qMessage);
            }
            if (psiStateNumber >= 4){
                S(qMessage);
            }

            // Teleport the state using Standard Teleportation
            Entangle(qAlice, qBob);
            let classicalBits = SendMessage(qAlice, qMessage);
            ReconstructMessage(qBob, classicalBits);   

            // Perform inverse operations on qBob qubit to verify success of teleportation
            if (psiStateNumber >= 4){
                S(qBob);
            }
            if (psiStateNumber >= 2){
                H(qBob);
            }
            if (psiStateNumber % 2 == 1){
                X(qBob);
            }

            if not CheckZero(qBob){
                Message($"The state {psiState} was not teleported successfully.");
            } else {
                Message($"Success!");
            }

            ResetAll([qAlice, qBob, qMessage]);
        }
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
