namespace Kata {
    open Microsoft.Quantum.Convert;

    function IsSeven(x : Bool[]) : Bool {
        return BoolArrayAsInt(x) == 7;
    }
}
