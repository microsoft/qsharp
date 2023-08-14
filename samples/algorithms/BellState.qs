/// # Sample
/// Bell States
///
/// # Description
/// Bell states or EPR pairs are specific quantum states of two qubits
/// that represent the simplest (and maximal) examples of quantum entanglement.
///
/// This Q# program implements the four different Bell states.
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    @EntryPoint()
    operation BellStates() : (Result, Result)[] {
        // Allocate the two qubits that will be used to create a Bell state.
        use register = Qubit[2];

        // This array contains a label and a preparation operation for each one
        // of the four Bell states.
        let bellStateTuples = [
            ("|Φ+〉", PreparePhiPlus),
            ("|Φ-〉", PreparePhiMinus),
            ("|Ψ+〉", PreparePsiPlus),
            ("|Ψ-〉", PreparePsiMinus)
        ];

        // Prepare all Bell states, show them using the `DumpMachine` operation
        // and measure the Bell state qubits.
        mutable measurements = [];
        for (label, prepare) in bellStateTuples {
            prepare(register);
            Message($"Bell state {label}:");
            DumpMachine();
            set measurements += [(M(register[0]), M(register[1]))];
            ResetAll(register);
        }
        return measurements;
    }

    operation PreparePhiPlus(register : Qubit[]) : Unit {
        ResetAll(register);             // |00〉
        H(register[0]);                 // |+0〉
        CNOT(register[0], register[1]); // 1/sqrt(2)(|00〉 + |11〉)
    }

    operation PreparePhiMinus(register : Qubit[]) : Unit {
        ResetAll(register);             // |00〉
        H(register[0]);                 // |+0〉
        Z(register[0]);                 // |-0〉
        CNOT(register[0], register[1]); // 1/sqrt(2)(|00〉 - |11〉)
    }

    operation PreparePsiPlus(register : Qubit[]) : Unit {
        ResetAll(register);             // |00〉
        H(register[0]);                 // |+0〉
        X(register[1]);                 // |+1〉
        CNOT(register[0], register[1]); // 1/sqrt(2)(|01〉 + |10〉)
    }

    operation PreparePsiMinus(register : Qubit[]) : Unit {
        ResetAll(register);             // |00〉
        H(register[0]);                 // |+0〉
        Z(register[0]);                 // |-0〉
        X(register[1]);                 // |-1〉
        CNOT(register[0], register[1]); // 1/sqrt(2)(|01〉 - |10〉)
    }
}
