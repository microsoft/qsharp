namespace Kata {
    operation Oracle_BitSumDivisibleBy3 (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        use counter = Qubit[2];
        within {
            for q in x {
                Controlled IncrementMod3([q], counter);
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
