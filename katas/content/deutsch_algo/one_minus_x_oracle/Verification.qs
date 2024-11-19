namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation PhaseOracle_OneMinusX_Reference(x : Qubit) : Unit is Adj + Ctl {
        Z(x);
        R(PauliI, 2.0 * PI(), x);
    }

    operation CheckSolution() : Bool {
        let solution = register => Kata.PhaseOracle_OneMinusX(register[0]);
        let reference = register => PhaseOracle_OneMinusX_Reference(register[0]);
        let isCorrect = CheckOperationsAreEqualStrict(1, solution, reference);

        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the effect your solution has on the state 0.6|0〉 + 0.8|1〉 and compare it with the effect it " +
                "is expected to have. Note that the simulator might drop the global phase -1, so if you're getting " +
                "verdict \"Incorrect\" but the actual state matches the expected one, check that you're handling the global phase correctly.");
            ShowQuantumStateComparison(1, (qs => Ry(ArcTan2(0.8, 0.6) * 2.0, qs[0])), solution, reference);
        }
        isCorrect
    }
}
