namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation CompoundGate(qs : Qubit[]) : Unit is Adj + Ctl {
        S(qs[0]);
        I(qs[1]); // this line can be omitted, since it doesn't change the qubit state
        Y(qs[2]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.CompoundGate;
        let reference = CompoundGate;
        let isCorrect = CheckOperationsAreEqualStrict(3, solution, reference);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                "transformation");
            ShowQuantumStateComparison(3, PrepDemoState, solution, reference);
        }

        isCorrect
    }
}
