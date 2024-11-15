namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ], Kata.DistinguishYfromXZ, ["Y", "XZ"], 1)
    }
}
