namespace Kata {
    operation Oracle_Periodic (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        // Implement your solution here...

    }

    // You might find this helper operation from an earlier task useful.
    operation Oracle_PeriodicGivenPeriod (x : Qubit[], y : Qubit, p : Int) : Unit is Adj + Ctl {
        let n = Length(x);
        within {
            for i in 0 .. n - p - 1 {
                CNOT(x[i + p], x[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, x[... n - p - 1], y);
        }
    }    
}
