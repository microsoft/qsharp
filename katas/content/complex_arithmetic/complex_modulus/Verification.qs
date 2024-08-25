namespace Kata.Verification {
    import Microsoft.Quantum.Convert.DoubleAsStringWithPrecision;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Convert;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();

            let expected = AbsComplex(x);
            let actual = Kata.ComplexModulus(x);
        
            if  AbsD(expected - actual) > 1e-6 {            
                // In case of an error, this value defines the precision with which complex numbers should be displayed
                let precision = 6;
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x,precision)} expected return {DoubleAsStringWithPrecision(expected,6)}, actual return {DoubleAsStringWithPrecision(actual,6)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
