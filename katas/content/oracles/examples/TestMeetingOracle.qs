namespace Kata {
    import Std.Arrays.*;
    import Std.Diagnostics.*;
    import Std.Convert.*;

    // The classical function to perform the same computation
    function Meeting_Classical(x : Bool[], jasmine : Bool[]) : Bool {
        for i in IndexRange(x) {
            if (not x[i]) and (not jasmine[i]) {
                // They have a day that they can both meet
                return true;
            }
        }
        return false;
    }

    operation Or_Oracle(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    operation Meeting_Oracle(x : Qubit[], jasmine : Qubit[], z : Qubit) : Unit is Adj + Ctl {
        use qs = Qubit[Length(x)];
        within {
            for i in IndexRange(qs) {
                // Flip q[i] if both you and Jasmine are free on the given day
                X(x[i]);
                X(jasmine[i]);
                CCNOT(x[i], jasmine[i], qs[i]);
            }
        } apply {
            Or_Oracle(qs, z);
        }
    }

    @EntryPoint()
    operation Test_Meeting_Oracle() : Unit {
        // There are 2^5 ways to arrange each of the schedules - let's try all of them
        for k in 0..2^5 - 1 {
            for j in 0..2^5 - 1 {
                // Convert your and Jasmine's schedules to bit arrays
                let binaryX = IntAsBoolArray(k, 5);
                let binaryJasmine = IntAsBoolArray(j, 5);

                // Calculate the function classically
                let classicalResult = Meeting_Classical(binaryX, binaryJasmine);

                // Allocate qubits to represent your schedule, Jasmine's schedule, and the output
                use (x, jasmine, target) = (Qubit[5], Qubit[5], Qubit());
                // Prepare the quantum schedules in basis states matching the binary schedules
                ApplyPauliFromBitString(PauliX, true, binaryX, x);
                ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);

                // Apply the quantum oracle
                Meeting_Oracle(x, jasmine, target);

                // Compare the result of the quantum oracle with that of the classical oracle
                if CheckZero(target) == classicalResult {
                    Message($"Failed on test case k = {k}, j = {j}. Classical result is not the same as quantum.");
                }

                // Undo the preparation of basis states x and jasmine
                ApplyPauliFromBitString(PauliX, true, binaryX, x);
                ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);

                // Check that the oracle did not change the input states
                if not CheckAllZero(x) {
                    Message($"Failed on test case k = {k}, j = {j}. Input state of x changed.");
                }
                if not CheckAllZero(jasmine) {
                    Message($"Failed on test case k = {k}, j = {j}. Input state of jasmine changed.");
                }

                Reset(target);
            }
        }
        Message("Success!");
    }
}
