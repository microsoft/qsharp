namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function ComplexConjugate_Reference(x : Complex) : Complex {
        // Return the complex conjugate  
        Complex(x::Real, -x::Imag)
    }    

    @EntryPoint()
    operation CheckSolution() : Bool {        
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();

            let expected = ComplexConjugate_Reference(x);
            let actual = Kata.ComplexConjugate(x);
        
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