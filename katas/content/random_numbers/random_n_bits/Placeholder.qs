namespace Kata {
    operation RandomNBits(N : Int) : Int {
        // Implement your solution here...

        return -1;
    }

    // You can use the operation defined in the first exercise to implement your solution.
    operation RandomBit() : Int {
        use q = Qubit();
        H(q);
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
