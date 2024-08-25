namespace Kata.Verification {
    function Inverse_Reference() : Double[][] {
        return [[-2., 1.], 
                [1.5, -0.5]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualD(Kata.Inverse(), Inverse_Reference(), precision)
    }
}
