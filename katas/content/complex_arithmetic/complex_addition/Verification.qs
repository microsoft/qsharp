namespace Kata.Verification {
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation CheckSolution() : Bool {            
        // In case of an error, this value defines the precision with which complex numbers should be displayed
        let precision = 6;
        CheckTwoComplexOpsAreSame(Kata.ComplexAdd, PlusC, precision)
    }
}
