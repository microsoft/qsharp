namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([qs => CNOT(qs[0], qs[1]), qs => CNOT(qs[1], qs[0])], Kata.CNOTDirection, ["CNOT_12", "CNOT_21"], 1)
    }
}
