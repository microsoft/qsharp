namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation CheckSolution() : Bool {
        for (n, oracle, expected, name) in [
            (2, qs => (), true, "f(x) = 0"),
            (3, qs => R(PauliI, 2.0 * PI(), qs[0]), true, "f(x) = 1"),
            (3, ApplyToEach(Z, _), false, "f(x) = parity of x"),
            (3, qs => Z(qs[0]), false, "f(x) = most significant bit of x"),
            (3, qs => Z(qs[Length(qs) - 1]), false, "f(x) = x mod 2"),
        ] {
            let actual = Kata.DeutschJozsaAlgorithm(n, oracle);
            if actual != expected {
                Message("Incorrect.");
                let actualStr = ConstantOrBalanced(actual);
                let expectedStr = ConstantOrBalanced(expected);
                Message($"{name} for {n} bits identified as {actualStr} but it is {expectedStr}.");
                return false;
            }
        }

        Message("Correct!");
        true
    }

    function ConstantOrBalanced(value : Bool) : String {
        return value ? "constant" | "balanced";
    }
}
