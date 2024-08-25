namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function InnerProduct_Reference() : Complex {
        return Complex(-18., 72.);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let actual = Kata.InnerProduct();
        let expected = InnerProduct_Reference();
        if AbsComplex(MinusC(actual, expected)) > 1e-9 {
            // In case of an error, this value defines the precision with which complex numbers should be displayed
            let precision = 2;
            Message("Incorrect");
            Message($"Expected {ComplexAsString(expected, precision)}, actual {ComplexAsString(actual, precision)}");
            return false;
        }
        
        Message("Correct!");
        return true;
    }
}
