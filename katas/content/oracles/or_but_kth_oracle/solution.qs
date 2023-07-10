namespace Kata.Verification {

    // Task 3.3.
    operation OrOfBitsExceptKth_Oracle_Reference (x : Qubit[], k : Int) : Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            Or_Oracle_Reference(x[...k-1] + x[k+1...], minus);
        }
    }

}
