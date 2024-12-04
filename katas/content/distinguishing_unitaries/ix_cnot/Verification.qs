namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([qs => X(qs[1]), qs => CNOT(qs[0], qs[1])], Kata.DistinguishIXfromCNOT, ["I⊗X", "CNOT"], 1)
    }
}
