namespace Kata.Verification {
    import Std.Math.*;

    function ComplexConjugate_Reference(x : Complex) : Complex {
        // Return the complex conjugate
        Complex(x.Real, -x.Imag)
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0..24 {
            let x = DrawRandomComplex();

            let expected = ComplexConjugate_Reference(x);
            let actual = Kata.ComplexConjugate(x);

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
