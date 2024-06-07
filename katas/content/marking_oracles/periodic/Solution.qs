namespace Kata {
    operation Oracle_Periodic (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        use aux = Qubit[N - 1];
        within {
            for P in 1 .. N - 1 {
                Oracle_PeriodicGivenPeriod(x, aux[P - 1], P);
            }
        } apply {
            ApplyControlledOnInt(0, X, aux, y);
            X(y);
        }
    }

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
