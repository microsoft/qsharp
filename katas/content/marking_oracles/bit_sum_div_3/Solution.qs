namespace Kata {
    open Microsoft.Quantum.Katas;
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
}
