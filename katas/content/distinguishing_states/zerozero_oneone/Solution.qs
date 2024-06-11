namespace Kata {
    operation ZeroZeroOrOneOne(qs : Qubit[]) : Int {
        return M(qs[0]) == One ? 1 | 0;
    }
}
