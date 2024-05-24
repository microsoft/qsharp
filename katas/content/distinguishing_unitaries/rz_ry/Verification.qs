namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Rz, Ry], Kata.DistinguishRzFromRy, ["Rz", "Ry"], 1)
    }
}
