namespace Kata {
    operation DeutschJozsaAlgorithm (N : Int, oracle : Qubit[] => Unit) : Bool {
        mutable isConstant = true;
        use x = Qubit[N];
        ApplyToEach(H, x);
        oracle(x);
        ApplyToEach(H, x);
        for xi in x {
            if MResetZ(xi) == One {
                set isConstant = false;
            }
        }
        return isConstant;
    }
}
