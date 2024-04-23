namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    operation  EntangleQubits (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
    }

    operation CheckOperationsEquivalenceOnInitialStateStrict(
        initialState : Qubit[] => Unit is Adj,
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int
    ) : Bool {
        use (control, target) = (Qubit(), Qubit[inputSize]);
        within {
            H(control);
            initialState(target);
        }
        apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }
        let isCorrect = CheckAllZero([control] + target);
        ResetAll([control] + target);
        isCorrect
    }
    
    operation CheckSolution() : Bool {
        let range = 10;
        for i in 0 .. range - 1 {
            let angle = 2.0 * PI() * IntAsDouble(i) / IntAsDouble(range);
            let initialState = qs => Ry(2.0 * angle, qs[0]);
            let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
                initialState,
                Kata.EntangleQubits, 
                EntangleQubits, 
                2);
            if not isCorrect {
                Message("Incorrect");
                Message($"Test fails for alpha = {Cos(angle)}, beta = {Sin(angle)}.");
                ShowQuantumStateComparison(2, initialState, Kata.EntangleQubits, EntangleQubits);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}