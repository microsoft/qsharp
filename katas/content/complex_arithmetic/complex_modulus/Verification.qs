namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for _ in 0 .. 24 {
            let x = DrawRandomComplex();

            let expected = AbsComplex(x);
            let actual = Kata.ComplexModulus(x);
        
            if  AbsD(expected - actual) > 1e-6 {
                Message("Incorrect");
                Message($"For x = {ComplexAsString(x)} expected return {expected}, actual return {actual}.");
                return false;
            }
        }

        Message("Correct!");
        return true;
    }
}
