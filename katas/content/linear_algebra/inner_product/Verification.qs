namespace Kata.Verification {
    import Std.Math.*;

    function InnerProduct_Reference() : Complex {
        return Complex(-18., 72.);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let actual = Kata.InnerProduct();
        let expected = InnerProduct_Reference();
        if AbsComplex(MinusC(actual, expected)) > 1e-9 {
            Message("Incorrect");
            Message($"Expected {ComplexAsString(expected)}, actual {ComplexAsString(actual)}");
            return false;
        }

        Message("Correct!");
        return true;
    }
}
