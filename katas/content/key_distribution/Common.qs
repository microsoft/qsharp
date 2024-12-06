namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Random.*;

    operation RandomArray(N : Int) : Bool[] {
        ForEach(x => DrawRandomInt(0, 1) == 0, [0, size = N])
    }

    operation BasisToString(base : Bool) : String {
        base ? "Hadamard" | "computational"
    }

    operation StateToString(base : Bool, bit : Bool) : String {
        if base {
            // ∣+⟩ / ∣-⟩
            return bit ? "|-⟩" | "|+⟩";
        } else {
            // ∣0⟩ / ∣1⟩
            return bit ? "|1⟩" | "|0⟩";
        }
    }

    operation PrepareQubits_Reference(qs : Qubit[], bases : Bool[], bits : Bool[]) : Unit is Adj {
        for i in 0..Length(qs) - 1 {
            if bits[i] {
                X(qs[i]);
            }
            if bases[i] {
                H(qs[i]);
            }
        }
    }
}
