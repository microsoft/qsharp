namespace Kata.Verification {
    function ScalarMultiplication_Reference() : Double[][] {
        return [[0.5, 1.], 
                [1.5, 2.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.ScalarMultiplication(), ScalarMultiplication_Reference())
    }
}
