namespace Kata {
    open Microsoft.Quantum.Katas;    
    operation Oracle_DivisibleBy3 (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        use counter = Qubit[2];
        within {
            for i in 0 .. Length(x) - 1 { // Iterate starting from the least significant bit
                if i % 2 == 0 {
                    // i-th power of 2 is 1 mod 3
                    Controlled IncrementMod3([x[i]], counter);
                } else {
                    // i-th power of 2 is 2 mod 3 - same as -1, which is Adjoint of +1
                    Controlled Adjoint IncrementMod3([x[i]], counter);
                }
            }
        } apply {
            // divisible by 3 only if the result is divisible by 3
            ApplyControlledOnInt(0, X, counter, y);
        }
    }
}
