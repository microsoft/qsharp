namespace Kata {
    open Microsoft.Quantum.Math;
    
    operation Oracle_Balanced (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        let log = BitSizeI(N);
        use inc = Qubit[log];
        within {
            for q in x {
                Controlled IncrementBE([q], inc);
            }
        } apply {
            ApplyControlledOnInt(N / 2, X, inc, y);
        }
    }

    operation IncrementBE (register : Qubit[]) : Unit is Adj + Ctl {
        if Length(register) > 1 {
            Controlled IncrementBE([register[0]], register[1 ...]);
        }
        X(register[0]);
    }    
}
