namespace Kata.Verification {
    import Std.Math.*;

    function NormalizedVector_Reference() : Complex[][] {
        return [
            [Complex(-0.6, 0.)],
            [Complex(0., 0.8)]
        ];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualC(Kata.NormalizedVector(), NormalizedVector_Reference())
    }
}
