namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([qs => CNOT(qs[0], qs[1]), qs => SWAP(qs[0], qs[1])], Kata.DistinguishCNOTfromSWAP, ["CNOT", "SWAP"], 1)
    }
}
