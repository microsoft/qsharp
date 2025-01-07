namespace Kata.Verification {
    import Std.Math.*;

    function ComplexEqual(x : Complex, y : Complex) : Bool {
        // Tests two complex numbers for equality.
        AbsD(x::Real - y::Real) <= 0.001 and AbsD(x::Imag - y::Imag) <= 0.001
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let actual = Kata.EigenvaluesS();
        let expected = [Complex(1.0, 0.0), Complex(0.0, 1.0)];
        if Length(actual) != 2 {
            Message("The array of eigenvalues should have exactly two elements.");
            return false;
        }
        if ComplexEqual(actual[0], expected[0]) and ComplexEqual(actual[1], expected[1]) or
            ComplexEqual(actual[0], expected[1]) and ComplexEqual(actual[1], expected[0]) {
            Message("Correct!");
            return true;
        }
        Message("Incorrect value for one of the eigenvalues.");
        return false;
    }
}
