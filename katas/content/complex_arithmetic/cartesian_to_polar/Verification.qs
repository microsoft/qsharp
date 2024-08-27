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
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)} expected return {ComplexPolarAsString(expected)}, actual return {ComplexPolarAsString(actual)}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}