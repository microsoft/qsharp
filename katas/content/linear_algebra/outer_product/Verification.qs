namespace Kata.Verification {
    import Std.Math.*;

    function OuterProduct_Reference() : Complex[][] {
        return [
            [Complex(-27., 0.), Complex(0., -6.)],
            [Complex(0., -81.), Complex(18., 0.)]
        ];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualC(Kata.OuterProduct(), OuterProduct_Reference())
    }
}
