namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0..24 {
            let x = DrawRandomComplex();
            let y = DrawRandomComplex();
            let xp = ComplexAsComplexPolar(x);
            let yp = ComplexAsComplexPolar(y);

            let expected = ComplexAsComplexPolar(TimesC(x, y));
            let actual = Kata.ComplexPolarMult(xp, yp);

            if not ComplexPolarEqual(expected, actual) {
                Message("Incorrect");
                Message($"For x = {ComplexPolarAsString(xp)}, y = {ComplexPolarAsString(yp)} " + 
                    $"expected return {ComplexPolarAsString(expected)}, actual return {ComplexPolarAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
