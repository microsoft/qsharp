namespace Kata.Verification {
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    operation PrepareBellState(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }


    operation BellStateChange1_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        Z(qs[0]);
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


    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
            PrepareBellState,
            Kata.BellStateChange1, 
            BellStateChange1_Reference, 
            2);


        if isCorrect {
            Message("Correct!");
        } else {
            ShowQuantumStateComparison(2, PrepareBellState, Kata.BellStateChange1, BellStateChange1_Reference);
        }


        return isCorrect;
    }

   
}
