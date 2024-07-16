namespace Kata {
    operation BernsteinVaziraniAlgorithm (N : Int, oracle : Qubit[] => Unit) : Int[] {
        mutable s = [];
        use x = Qubit[N];
        ApplyToEach(H, x);
        oracle(x);
        ApplyToEach(H, x);
        for xi in x {
            set s += [MResetZ(xi) == Zero ? 0 | 1];
        }
        return s;
    }
}
