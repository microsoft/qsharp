namespace Kata {
    operation Oracle_DivisibleBy3 (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        use counter = Qubit[2];
        within {
            for i in 0 .. Length(x) - 1 {
                if i % 2 == 0 {
                    Controlled IncrementMod3([x[i]], counter);
                } else {
                    Controlled Adjoint IncrementMod3([x[i]], counter);
                }
            }
        } apply {
            ApplyControlledOnInt(0, X, counter, y);
        }
    }

    operation IncrementMod3 (counterRegister : Qubit[]) : Unit is Adj + Ctl {
        let sum = counterRegister[0];
        let carry = counterRegister[1];
        ApplyControlledOnInt(0, X, [carry], sum);
        ApplyControlledOnInt(0, X, [sum], carry);
    }    
}
