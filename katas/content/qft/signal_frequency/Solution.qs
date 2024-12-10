namespace Kata {
    import Std.Arrays.*;

    operation SignalFrequency(qs : Qubit[]) : Int {
        Adjoint QFT(qs);
        return MeasureInteger(Reversed(qs));
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
