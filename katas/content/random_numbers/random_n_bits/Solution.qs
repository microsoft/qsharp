namespace Kata {
    operation RandomNBits(N : Int) : Int {
        mutable result = 0;
        for i in 0 .. N - 1 {
            set result = result * 2 + RandomBit();
        }
        return result;
    }

    // You can use the operation defined in the first exercise to implement your solution.
    operation RandomBit() : Int {
        use q = Qubit();
        H(q);
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
