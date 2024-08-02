namespace Kata {
    open Microsoft.Quantum.Arrays;

    operation SignalFrequency(qs : Qubit[]) : Int {
        // Implement your solution here...
        
        return -1;
    }

    // You might find this helper operation that implements QFT using a library operation useful.
    operation QFT(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyQFT(Reversed(qs));
        SwapReverseRegister(qs);
    }
}
