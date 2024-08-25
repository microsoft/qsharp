namespace Kata.Verification {
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;
    open Microsoft.Quantum.Convert;    
    
    function ComplexExpReal_Reference(r : Double, x : Complex) : Complex {
        if AbsD(r) < 1e-9 {
            return Complex(0.0, 0.0);
        }
        let real = r ^ x::Real * Cos(x::Imag * Log(r));
        let imaginary = r ^ x::Real * Sin(x::Imag * Log(r));
        return Complex(real, imaginary);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for ind in 0 .. 24 {
            let x = DrawRandomComplex();
            let r = ind == 0 ? 0.0 | DrawRandomDouble(0., 10.);

            let expected = ComplexExpReal_Reference(r, x); 
            let actual = Kata.ComplexExpReal(r, x);        
        
            if not ComplexEqual(expected, actual) {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x, precision)} and r = {DoubleAsStringWithPrecision(r,6)} expected return {ComplexAsString(expected, precision)}, actual return {ComplexAsString(actual, precision)}.");
                return false;
            }                
        }            

        Message("Correct!");
        return true;        
    }
}
