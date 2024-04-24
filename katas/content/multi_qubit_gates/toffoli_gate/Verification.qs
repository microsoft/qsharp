namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    operation PrepareState(angle:Double,qs : Qubit[]) : Unit is Adj + Ctl {
        for i in 0 .. Length(qs)-1{
            Ry(angle,qs[i]);
        }
    }

    operation  ToffoliGate (qs : Qubit[]) : Unit is Adj + Ctl {
        CCNOT(qs[0], qs[1], qs[2]);
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
            let initialState = qs => PrepareState(2.0 *angle,qs);
            let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
                initialState,
                Kata.ToffoliGate, 
                ToffoliGate, 
                3);
            if not isCorrect {
                Message("Incorrect");
                Message($"Test fails for alpha = {Cos(angle)}, beta = {Sin(angle)}.");
                ShowQuantumStateComparison(3, initialState, Kata.ToffoliGate, ToffoliGate);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}