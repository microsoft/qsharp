namespace Kata {
    import Std.Math.*;

    operation Oracle_Majority(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        let log = BitSizeI(N);
        use inc = Qubit[log];
        within {
            for q in x {
                Controlled Increment([q], inc);
            }
        } apply {
            CNOT(inc[log-1], y);
            if N == 5 {
                CCNOT(inc[0], inc[1], y);
            }
        }
    }

    operation Increment(register : Qubit[]) : Unit is Adj + Ctl {
        if Length(register) > 1 {
            Controlled Increment([register[0]], register[1...]);
        }
        X(register[0]);
    }
}
