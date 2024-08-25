namespace Kata.Verification {
    function Addition_Reference() : Double[][] {
        return [[6., 8.], 
                [10., 12.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 2;
        ArraysEqualD(Kata.Addition(), Addition_Reference(), precision)
    }
}
