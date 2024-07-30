namespace Kata {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    operation BinaryFractionClassical(q : Qubit, j : Bool[]) : Unit is Adj + Ctl {
        R1(2.0 * PI() * IntAsDouble(BoolArrayAsInt(Reversed(j))) / IntAsDouble(1 <<< Length(j)), q);
    }
}