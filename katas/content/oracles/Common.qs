namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;

    operation ApplyOracle (qs : Qubit[], oracle : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : Unit is Adj + Ctl {
        let N = Length(qs);
        oracle(qs[0 .. N - 2], qs[N - 1]);
    }

    operation PrepRandomState(qs : Qubit[]) : Unit {
        for q in qs {
            Ry(DrawRandomDouble(0.01, 0.99) * 2.0, q);
        }
    }
}
