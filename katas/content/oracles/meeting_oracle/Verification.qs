namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;

    operation Or_Oracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    operation Meeting_Oracle_Reference(x : Qubit[], jasmine : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        use qs = Qubit[Length(x)];
        within {
            for i in IndexRange(qs) {
                // flip q[i] if both x and jasmine are free on the given day
                X(x[i]);
                X(jasmine[i]);
                CCNOT(x[i], jasmine[i], qs[i]);
            }
        } apply {
            Or_Oracle_Reference(qs, y);
        }
    }

    operation ApplyMeetingOracle(qs : Qubit[], oracle : (Qubit[], Qubit[], Qubit) => Unit is Adj + Ctl) : Unit is Adj + Ctl {
        let x = qs[0..4];
        let jasmine = qs[5..9];
        let target = qs[10];
        oracle(x, jasmine, target);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let N = 5;
        let sol = ApplyMeetingOracle(_, Kata.Meeting_Oracle);
        let ref = ApplyMeetingOracle(_, Meeting_Oracle_Reference);
        let isCorrect = CheckOperationsAreEqualStrict(2 * N + 1, sol, ref);

        if not isCorrect {
            Message("Incorrect.");
            Message("Hint: check that you're flipping the state of the target qubit for the correct inputs, " +
                "that you're uncomputing any changes you did to the input qubits correctly, " +
                "and that you're returning any temporarily allocated qubits to the zero state.");
            return false;
        }
        Message("Correct!");
        true
    }
}
