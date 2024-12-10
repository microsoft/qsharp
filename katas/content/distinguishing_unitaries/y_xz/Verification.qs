namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ], Kata.DistinguishYfromXZ, ["Y", "XZ"], 1)
    }
}
