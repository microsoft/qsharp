namespace Kata.Verification {

    open Microsoft.Quantum.Arrays;

    // Task 4.2.
    operation ArbitraryBitPattern_Oracle_Challenge(x: Qubit[], pattern: Bool[]): Unit is Adj + Ctl {
        within {
            for i in IndexRange(x) {
                if not pattern[i] {
                    X(x[i]);
                }
            }
        } apply {
            Controlled Z(Most(x), Tail(x));
        }
    }

}
