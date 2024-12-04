namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Convert;

    operation DrawRandomComplex() : Complex {
        // Generates a random complex number.
        let real = DrawRandomDouble(-10., 10.);
        let imag = DrawRandomDouble(-10., 10.);
        return Complex(real, imag);
    }

    function ComplexAsString(x : Complex) : String {
        let precision = 3;
        if x.Imag < 0.0 {
            $"{DoubleAsStringWithPrecision(x.Real, precision)} - {DoubleAsStringWithPrecision(AbsD(x.Imag), precision)}i"
        } else {
            $"{DoubleAsStringWithPrecision(x.Real, precision)} + {DoubleAsStringWithPrecision(x.Imag, precision)}i"
        }
    }

    function ComplexPolarAsString(x : ComplexPolar) : String {
        let precision = 3;
        $"{DoubleAsStringWithPrecision(x.Magnitude, precision)} * exp({DoubleAsStringWithPrecision(x.Argument, precision)}i)"
    }

    operation CheckTwoComplexOpsAreSame(sol : (Complex, Complex) -> Complex, ref : (Complex, Complex) -> Complex) : Bool {
        for _ in 0..24 {
            let x = DrawRandomComplex();
            let y = DrawRandomComplex();

            let expected = ref(x, y);
            let actual = sol(x, y);

            if not ComplexEqual(expected, actual) {
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)}, y = {ComplexAsString(y)} expected return {ComplexAsString(expected)}, actual return {ComplexAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }

    function ComplexEqual(x : Complex, y : Complex) : Bool {
        // Tests two complex numbers for equality.
        AbsD(x.Real - y.Real) <= 0.001 and AbsD(x.Imag - y.Imag) <= 0.001
    }

    function ComplexPolarEqual(x : ComplexPolar, y : ComplexPolar) : Bool {
        // Tests two complex polar numbers for equality.
        AbsD(x.Magnitude - y.Magnitude) <= 0.001 and AbsD(x.Argument - y.Argument) <= 0.001
    }
}
