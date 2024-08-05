namespace Kata {
    open Microsoft.Quantum.Arrays;

    operation AllBasisVectors(qs : Qubit[]) : Unit is Adj + Ctl {
        QFT(qs);
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
