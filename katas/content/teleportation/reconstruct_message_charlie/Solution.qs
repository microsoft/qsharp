namespace Kata {
    operation ReconstructMessageWhenThreeEntangledQubits(qCharlie : Qubit, (b1 : Bool, b2 : Bool), b3 : Bool) : Unit {
        if b1 {
            Z(qCharlie);
        }
        if b2 != b3 {
            X(qCharlie);
        }
    }
}