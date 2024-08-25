namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {        
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();

            let expected = ComplexAsComplexPolar(x);
            let actual = Kata.ComplexToComplexPolar(x);
        
            if not ComplexPolarEqual(expected, actual) {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x,precision)} expected return {ComplexPolarAsString(expected,precision)}, actual return {ComplexPolarAsString(actual,precision)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}