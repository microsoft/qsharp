namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([ApplyToEachCA(I, _), qs => CNOT(qs[0], qs[1]), qs => CNOT(qs[1], qs[0]), qs => SWAP(qs[0], qs[1])], Kata.DistinguishTwoQubitUnitaries, ["I⊗I", "CNOT_12", "CNOT_21", "SWAP"], 2)
    }
}
