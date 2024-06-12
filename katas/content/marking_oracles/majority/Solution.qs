namespace Kata {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    operation Oracle_Majority (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        let log = BitSizeI(N);
        use inc = Qubit[log];
        within {
            for q in x {
                Controlled IncrementBE([q], inc);
            }
        } apply {
            CNOT(inc[log-1], y);
            if log > 2 {
                CCNOT(inc[0], inc[1], y);
            }
        }
    } 
}
