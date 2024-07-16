namespace Kata {
    operation GHZOrWState(qs : Qubit[]) : Int {
        mutable countOnes = 0;

        for q in qs {
            if M(q) == One {
                set countOnes += 1;
            }
        }

        return countOnes == 1 ? 1 | 0;
    }
}
