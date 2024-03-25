namespace Kata.Verification {
    function Transpose_Reference() : Double[][] {
        return [[1., 3.], 
                [2., 4.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.Transpose(), Transpose_Reference())
    }
}
