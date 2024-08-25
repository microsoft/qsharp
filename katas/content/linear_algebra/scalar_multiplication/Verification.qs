namespace Kata.Verification {
    function ScalarMultiplication_Reference() : Double[][] {
        return [[0.5, 1.], 
                [1.5, 2.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualD(Kata.ScalarMultiplication(), ScalarMultiplication_Reference(), precision)
    }
}
