namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation CheckSolution() : Bool {
        for (oracle, expected, name) in [
            (I, true, "f(x) = 0"),
            (R(PauliI, 2.0 * PI(), _), true, "f(x) = 1"),
            (Z, false, "f(x) = x"),
            (PhaseOracle_OneMinusX, false, "f(x) = 1 - x")
        ] {

            let actual = Kata.DeutschAlgorithm(oracle);
            if actual != expected {
                Message("Incorrect.");
                let actualStr = ConstantOrVariable(actual);
                let expectedStr = ConstantOrVariable(expected);
                Message($"{name} identified as {actualStr} but it is {expectedStr}.");
                return false;
            }
        }

        Message("Correct!");
        true
    }

    function ConstantOrVariable(value : Bool) : String {
        return value ? "constant" | "variable";
    }

    operation PhaseOracle_OneMinusX(x : Qubit) : Unit is Adj + Ctl {
        Z(x);
        R(PauliI, 2.0 * PI(), x);
    }
}
