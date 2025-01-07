namespace Kata {
    import Std.Arrays.*;
    import Std.Convert.*;

    operation PeriodicState(qs : Qubit[], F : Int) : Unit is Adj + Ctl {
        let bitsBE = Reversed(IntAsBoolArray(F, Length(qs)));
        ApplyPauliFromBitString(PauliX, true, bitsBE, qs);
        QFT(qs);
    }

    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
