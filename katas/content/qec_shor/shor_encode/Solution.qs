namespace Kata {
    operation ShorEncode (qs : Qubit[]) : Unit is Adj + Ctl {
        BitflipEncode(qs[0 .. 3 .. 8]);
        ApplyToEachCA(H, qs[0 .. 3 .. 8]);
        for i in 0 .. 2 {
            BitflipEncode(qs[3 * i .. 3 * i + 2]);
        }
    }

    operation BitflipEncode (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
    }
}