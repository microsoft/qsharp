namespace Kata.Verification {
    function Inverse_Reference() : Double[][] {
        return [[-2., 1.], 
                [1.5, -0.5]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.Inverse(), Inverse_Reference())
    }
}
