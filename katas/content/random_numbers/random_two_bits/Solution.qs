namespace Kata {
    operation RandomTwoBits() : Int {
        return 2 * RandomBit() + RandomBit();
    }

    operation RandomBit() : Int {
        use q = Qubit();
        H(q);
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
