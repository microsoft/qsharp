namespace Kata.Verification {
    function Transpose_Reference() : Double[][] {
        return [[1., 3.], 
                [2., 4.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualD(Kata.Transpose(), Transpose_Reference(), precision)
    }
}
