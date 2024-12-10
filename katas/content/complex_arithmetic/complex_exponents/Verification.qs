namespace Kata.Verification {
    import Std.Math.*;

    function ComplexExponent_Reference(x : Complex) : Complex {
        let expa = E()^x.Real;
        return Complex(expa * Cos(x.Imag), expa * Sin(x.Imag));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0..24 {
            let x = DrawRandomComplex();

            let expected = ComplexExponent_Reference(x);
            let actual = Kata.ComplexExponent(x);

            if not ComplexEqual(expected, actual) {
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)} expected return {ComplexAsString(expected)}, actual return {ComplexAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
