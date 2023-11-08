namespace Test {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;

    operation PrepareEntangledState (
        left : Qubit[],
        right : Qubit[]) : Unit is Adj + Ctl {

        for idxQubit in 0 .. Length(left) - 1
        {
            H(left[idxQubit]);
            Controlled X([left[idxQubit]], right[idxQubit]);
        }
    }

    operation AssertOperationsEqualReferenced (
        nQubits : Int,
        actual : (Qubit[] => Unit),
        expected : (Qubit[] => Unit is Adj)) : Unit {

        // Prepare a reference register entangled with the target register.
        use (reference, target) = (Qubit[nQubits], Qubit[nQubits]) {
            PrepareEntangledState(reference, target);
            actual(target);
            Adjoint expected(target);
            Adjoint PrepareEntangledState(reference, target);
            let isCorrect = CheckAllZero(reference + target);
            ResetAll(target);
            ResetAll(reference);
            Fact(isCorrect, "OPERATIONS ARE NOT THE SAME!");
        }
    }

    /// # Summary
    /// Hard-code 1 qubit QFT
    operation QFT1 (target : Qubit[]) : Unit is Adj {
        Fact(Length(target) == 1, $"`Length(target!)` must be 1");
        H((target)[0]);
    }

    /// # Summary
    /// Hard-code 2 qubit QFT
    operation QFT2 (target : Qubit[]) : Unit is Adj {
        Fact(Length(target) == 2, $"`Length(target!)` must be 2");
        let (q1, q2) = ((target)[0], (target)[1]);
        H(q1);
        Controlled R1Frac([q2], (2, 2, q1));
        H(q2);
    }

    /// # Summary
    /// Hard-code 3 qubit QFT
    operation QFT3 (target : Qubit[]) : Unit is Adj {
        Fact(Length(target) == 3, $"`Length(target)` must be 3");
        let (q1, q2, q3) = ((target)[0], (target)[1], (target)[2]);
        H(q1);
        Controlled R1Frac([q2], (2, 2, q1));
        Controlled R1Frac([q3], (2, 3, q1));
        H(q2);
        Controlled R1Frac([q3], (2, 2, q2));
        H(q3);
    }

    /// # Summary
    /// Hard-code 4 qubit QFT
    operation QFT4 (target : Qubit[]) : Unit is Adj {
        Fact(Length(target) == 4, $"`Length(target!)` must be 4");
        let (q1, q2, q3, q4) = ((target)[0], (target)[1], (target)[2], (target)[3]);
        H(q1);
        Controlled R1Frac([q2], (2, 2, q1));
        Controlled R1Frac([q3], (2, 3, q1));
        Controlled R1Frac([q4], (2, 4, q1));
        H(q2);
        Controlled R1Frac([q3], (2, 2, q2));
        Controlled R1Frac([q4], (2, 3, q2));
        H(q3);
        Controlled R1Frac([q4], (2, 2, q3));
        H(q4);
    }

    /// # Summary
    /// Compares QFT to the hard-coded implementations
    operation TestQFT(n: Int) : Unit {
        Fact(n>=1 and n<=4, "Only have four tests for QFT.");
        let testOperations = [QFT1, QFT2, QFT3, QFT4];
        AssertOperationsEqualReferenced(n, testOperations[n-1], q => ApplyQFT(Reversed(q)));
    }
}
