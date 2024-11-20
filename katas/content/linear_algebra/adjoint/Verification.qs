namespace Kata.Verification {
    import Std.Math.*;

    function MatrixAdjoint_Reference() : Complex[][] {
        return [
            [Complex(1., -5.), Complex(3., 6.)],
            [Complex(2., 0.), Complex(0., -4.)]
        ];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualC(Kata.MatrixAdjoint(), MatrixAdjoint_Reference())
    }
}
