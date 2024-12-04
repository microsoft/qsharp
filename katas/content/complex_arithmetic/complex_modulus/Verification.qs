namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0..24 {
            let x = DrawRandomComplex();

            let expected = AbsComplex(x);
            let actual = Kata.ComplexModulus(x);

            if AbsD(expected - actual) > 1e-6 {
                let precision = 3;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)} expected return {DoubleAsStringWithPrecision(expected, precision)}, actual return {DoubleAsStringWithPrecision(actual, precision)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
