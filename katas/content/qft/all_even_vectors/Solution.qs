namespace Kata {
    import Std.Arrays.*;

    operation AllEvenVectors(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        QFT(qs);
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
