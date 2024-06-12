namespace Kata {
    operation Oracle_DivisibleBy3 (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        // Implement your solution here...

    }

    // Helper operation that implements counting modulo 3
    operation IncrementMod3 (counterRegister : Qubit[]) : Unit is Adj + Ctl {
        let sum = counterRegister[0];
        let carry = counterRegister[1];
        ApplyControlledOnInt(0, X, [carry], sum);
        ApplyControlledOnInt(0, X, [sum], carry);
    }
}
