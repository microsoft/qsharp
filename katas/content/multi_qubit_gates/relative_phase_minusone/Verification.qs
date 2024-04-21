namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Diagnostics;

    operation PrepareState(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyToEach(H, qs)
    }
    operation  RelativePhaseMinusOne (qs : Qubit[]) : Unit is Adj + Ctl {
        CZ(qs[0], qs[1]);
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
        let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
            PrepareState,
            Kata.RelativePhaseMinusOne, 
            RelativePhaseMinusOne, 
            2);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect");
            ShowQuantumStateComparison(2, PrepareState, Kata.RelativePhaseMinusOne, RelativePhaseMinusOne);
        }

        return isCorrect;
    }
}