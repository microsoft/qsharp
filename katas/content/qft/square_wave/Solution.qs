namespace Kata {
    import Std.Arrays.*;

    operation SquareWave(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        H(qs[0]);
        T(qs[0]);
        X(qs[0]);
        Adjoint T(qs[0]);
        X(qs[0]);
        QFT(qs);
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
