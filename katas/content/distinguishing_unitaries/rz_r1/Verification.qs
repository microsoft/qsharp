namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Rz, R1], Kata.DistinguishRzFromR1, ["Rz", "R1"], 1)
    }
}
