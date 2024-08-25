namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    function NormalizedVector_Reference() : Complex[][] {
        return [[Complex(-0.6, 0.)],
                [Complex(0., 0.8)]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualC(Kata.NormalizedVector(), NormalizedVector_Reference(), precision)
    }
}
