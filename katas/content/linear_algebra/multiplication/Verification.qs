namespace Kata.Verification {
    function Multiplication_Reference() : Double[][] {
        return [[19., 22.], 
                [43., 50.]];
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.Multiplication(), Multiplication_Reference())
    }
}
