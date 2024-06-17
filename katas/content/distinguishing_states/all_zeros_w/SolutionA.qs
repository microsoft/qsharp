namespace Kata {
    operation AllZerosOrWState(qs : Qubit[]) : Int {
        mutable countOnes = 0;

        for q in qs {
            if M(q) == One {
                set countOnes += 1;
            }
        }

        return countOnes == 0 ? 0 | 1;
    }
}
