namespace Kata.Verification {

    // Task 3.1.
    operation Or_Oracle(x: Qubit[], y: Qubit): Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

}
