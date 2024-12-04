namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, MinusXZ, MinusY, XZ], Kata.DistinguishYfromXZWithPhases, ["Y", "-XZ", "-Y", "XZ"], 1)
    }
}
