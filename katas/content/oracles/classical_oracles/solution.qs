namespace Kata.Verification {

    open Microsoft.Quantum.Convert;

    // Task 1.1.
    function IsSeven(x: Bool[]): Bool {
        return BoolArrayAsInt(x) == 7;
    }

}
