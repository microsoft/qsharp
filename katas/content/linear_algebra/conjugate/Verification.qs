namespace Kata.Verification {
    import Std.Math.*;

    function Conjugate_Reference() : Complex[][] {
        return [
            [Complex(1., -5.), Complex(2., 0.)],
            [Complex(3., 6.), Complex(0., -4.)]
        ];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualC(Kata.Conjugate(), Conjugate_Reference())
    }
}
