namespace Kata {
    import Std.Arrays.*;

    operation AlternatingAmplitudes(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        QFT(qs);
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
