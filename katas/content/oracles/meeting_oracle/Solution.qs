namespace Kata {
    import Std.Arrays.*;

    operation Or_Oracle(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    operation Meeting_Oracle(x : Qubit[], jasmine : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        use qs = Qubit[Length(x)];
        within {
            for i in IndexRange(qs) {
                // flip q[i] if both x and jasmine are free on the given day
                X(x[i]);
                X(jasmine[i]);
                CCNOT(x[i], jasmine[i], qs[i]);
            }
        } apply {
            Or_Oracle(qs, y);
        }
    }
}
