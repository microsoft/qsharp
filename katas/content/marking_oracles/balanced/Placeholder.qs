namespace Kata {
    operation Oracle_Balanced (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        // Implement your solution here...

    }

    // Helper operation that implements increment for a qubit register
    operation IncrementBE (register : Qubit[]) : Unit is Adj + Ctl {
        if Length(register) > 1 {
            Controlled IncrementBE([register[0]], register[1 ...]);
        }
        X(register[0]);
    }
}
