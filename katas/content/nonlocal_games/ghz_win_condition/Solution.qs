namespace Kata {
    open Microsoft.Quantum.Logical;

    function WinCondition (rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == Xor(Xor(abc[0], abc[1]), abc[2]);
    }
}
