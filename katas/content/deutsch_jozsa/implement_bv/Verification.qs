namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation CheckSolution() : Bool {
        for (n, oracle, expected, name) in [
            (2, qs => (), [0, 0], "f(x) = 0"),
            (3, qs => (), [0, 0, 0], "f(x) = 0"),
            (2, ApplyToEach(Z, _), [1, 1], "f(x) = parity of x"),
            (3, ApplyToEach(Z, _), [1, 1, 1], "f(x) = parity of x"),
            (2, qs => Z(qs[0]), [1, 0], "f(x) = most significant bit of x"),
            (3, qs => Z(qs[2]), [0, 0, 1], "f(x) = least significant bit of x"),
            (3, qs => Z(qs[1]), [0, 1, 0], "f(x) = middle bit of x")
        ] {
            let actual = Kata.BernsteinVaziraniAlgorithm(n, oracle);
            if actual != expected {
                Message("Incorrect.");
                Message($"The bit string for {name} for {n} bits identified as {actual} but it is {expected}.");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
