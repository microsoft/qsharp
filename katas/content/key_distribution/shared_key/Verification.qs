namespace Kata.Verification {
    import Std.Convert.*;

    function GenerateSharedKey_Reference(basesAlice : Bool[], basesBob : Bool[], bits : Bool[]) : Bool[] {
        mutable key = [];
        for i in 0..Length(bits) - 1 {
            if basesAlice[i] == basesBob[i] {
                set key += [bits[i]];
            }
        }

        return key;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        for N in 8..20 {
            let basesAlice = RandomArray(N);
            let basesBob = RandomArray(N);
            let bits = RandomArray(N);
            let expected = GenerateSharedKey_Reference(basesAlice, basesBob, bits);
            let result = Kata.GenerateSharedKey(basesAlice, basesBob, bits);

            if Length(result) != Length(expected) or BoolArrayAsInt(result) != BoolArrayAsInt(expected) {
                Message($"Unexpected key value");
                Message($"Alice's bases: {basesAlice}");
                Message($"Bob's bases: {basesBob}");
                Message($"Got key {result}, expected {expected}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
