namespace Kata {
    function GenerateSharedKey(basesAlice : Bool[], basesBob : Bool[], bits : Bool[]) : Bool[] {
        mutable key = [];
        for i in 0 .. Length(bits) - 1 {
            if basesAlice[i] == basesBob[i] {
                set key += [bits[i]];
            }
        }

        return key;
    }
}
