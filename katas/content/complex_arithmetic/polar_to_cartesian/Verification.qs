namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0..24 {
            let x = ComplexAsComplexPolar(DrawRandomComplex());

            let expected = ComplexPolarAsComplex(x);
            let actual = Kata.ComplexPolarToComplex(x);

            if not ComplexEqual(expected, actual) {
                Message("Incorrect");
                Message($"For x = {ComplexPolarAsString(x)} expected return {ComplexAsString(expected)}, actual return {ComplexAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
