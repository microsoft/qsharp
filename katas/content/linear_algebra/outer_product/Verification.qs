namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function OuterProduct_Reference() : Complex[][] {
        return [[Complex(-27., 0.), Complex(0., -6.)],
                [Complex(0., -81.), Complex(18., 0.)]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualC(Kata.OuterProduct(), OuterProduct_Reference(), precision)
    }
}
