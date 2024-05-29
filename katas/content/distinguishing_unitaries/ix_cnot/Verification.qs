namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([qs => X(qs[1]), qs => CNOT(qs[0], qs[1])], Kata.DistinguishIXfromCNOT, ["IX", "CNOT"], 1)
    }
}
