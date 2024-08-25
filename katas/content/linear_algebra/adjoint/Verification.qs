namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function MatrixAdjoint_Reference() : Complex[][] {
        return [[Complex(1., -5.), Complex(3., 6.)],
                [Complex(2., 0.), Complex(0., -4.)]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualC(Kata.MatrixAdjoint(), MatrixAdjoint_Reference(), precision)
    }
}
