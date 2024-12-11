namespace Kata {
    import Std.Arrays.*;

    operation ArbitraryBitPattern_Oracle_Challenge(
        x : Qubit[],
        pattern : Bool[]
    ) : Unit is Adj + Ctl {
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
