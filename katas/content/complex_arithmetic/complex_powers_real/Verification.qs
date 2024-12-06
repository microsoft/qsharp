namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Math.*;
    import Std.Random.*;

    function ComplexExpReal_Reference(r : Double, x : Complex) : Complex {
        if AbsD(r) < 1e-9 {
            return Complex(0., 0.);
        }
        let real = r^x.Real * Cos(x.Imag * Log(r));
        let imaginary = r^x.Real * Sin(x.Imag * Log(r));
        return Complex(real, imaginary);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for ind in 0..24 {
            let x = DrawRandomComplex();
            let r = ind == 0 ? 0.0 | DrawRandomDouble(0., 10.);

            let expected = ComplexExpReal_Reference(r, x);
            let actual = Kata.ComplexExpReal(r, x);

            if not ComplexEqual(expected, actual) {
                let precision = 3;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)} and r = {DoubleAsStringWithPrecision(r, precision)} expected return {ComplexAsString(expected)}, actual return {ComplexAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
