namespace Kata.Verification {        
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();
            let y = DrawRandomComplex();
            let xp = ComplexAsComplexPolar(x);
            let yp = ComplexAsComplexPolar(y);

            let expected = ComplexAsComplexPolar(TimesC(x, y));
            let actual = Kata.ComplexPolarMult(xp, yp);
        
            if not ComplexPolarEqual(expected, actual) {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexPolarAsString(xp, precision)}, y = {ComplexPolarAsString(yp, precision)} " + 
                    $"expected return {ComplexPolarAsString(expected, precision)}, actual return {ComplexPolarAsString(actual, precision)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}