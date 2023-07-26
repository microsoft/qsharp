namespace Kata {

    operation Or_Oracle(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    // Task 3.3.
    operation OrOfBitsExceptKth_Oracle(x: Qubit[], k: Int): Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            Or_Oracle(x[...k-1] + x[k+1...], minus);
        }
    }

}
