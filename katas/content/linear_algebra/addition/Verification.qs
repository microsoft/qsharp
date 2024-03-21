namespace Kata.Verification {
    operation Addition_Reference() : Double[][] {
        return [[6., 8.], 
                [10., 12.]];
   }

    @EntryPoint()
    operation CheckSolution() : Bool {
        ArraysEqualD(Kata.Addition(), Addition_Reference())
    }
}
