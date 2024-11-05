namespace Kata {
    operation Oracle_Palindrome (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        within {
            for i in 0 .. N / 2 - 1 {
                CNOT(x[N - 1 - i], x[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, x[... N / 2 - 1], y);
        }
    }    
}
