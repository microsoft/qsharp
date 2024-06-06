namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation SetQubitZeroOrPlus (q : Qubit, state : Int) : Unit {
        if state != 0 {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution () : Bool {
       return DistinguishStates_MultiQubit_Threshold(1, 2, 0.8, SetQubitZeroOrPlus, Kata.IsQubitZeroOrPlus);
    }
}
