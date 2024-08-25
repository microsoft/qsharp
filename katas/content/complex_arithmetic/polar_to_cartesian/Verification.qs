namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {        
        for _ in 0 .. 24 {
            let x = ComplexAsComplexPolar(DrawRandomComplex());

            let expected = ComplexPolarAsComplex(x);
            let actual = Kata.ComplexPolarToComplex(x);
        
            if not ComplexEqual(expected, actual) {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexPolarAsString(x, precision)} expected return {ComplexAsString(expected, precision)}, actual return {ComplexAsString(actual, precision)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}