namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    operation CheckSolution() : Bool {
        for (oracle, expected, name) in [(I, true, "f(x) = 0"), 
                                         (R(PauliI, 2.0 * PI(), _), true, "f(x) = 1"), 
                                         (Z, false, "f(x) = x"), 
                                         (PhaseOracle_OneMinusX, false, "f(x) = 1 - x")] {

            let actual = Kata.DeutschAlgorithm(oracle);
            if actual != expected {
                Message("Incorrect.");
                let actualStr = ConstantOrBalanced(actual);
                let expectedStr = ConstantOrBalanced(expected);
                Message($"{name} identified as {actualStr} but it is {expectedStr}.");
                return false;
            }
        }

        Message("Correct!");
        true
    }

    function ConstantOrBalanced (value : Bool) : String {
        return value ? "constant" | "balanced";
    }

    operation PhaseOracle_OneMinusX(x : Qubit) : Unit is Adj + Ctl {
        Z(x);
        R(PauliI, 2.0 * PI(), x);
    }
}
