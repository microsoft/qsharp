namespace Kata {
    import Std.Arrays.*;

    operation AllBasisVectors(qs : Qubit[]) : Unit is Adj + Ctl {
        // Implement your solution here...

    }

    // You might find this helper operation that implements QFT using a library operation useful.
    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
