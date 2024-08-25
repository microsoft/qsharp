namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;    
    
    function ComplexExponent_Reference(x : Complex) : Complex {
        let expa = E() ^ x::Real;
        return Complex(expa * Cos(x::Imag), expa * Sin(x::Imag));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();

            let expected = ComplexExponent_Reference(x);
            let actual = Kata.ComplexExponent(x);
        
            if not ComplexEqual(expected, actual) {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x,precision)} expected return {ComplexAsString(expected,precision)}, actual return {ComplexAsString(actual,precision)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
