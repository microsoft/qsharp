namespace Quantum.Kata.Reference {

    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Convert;

    // The classical function to perform the same computation
    function Meeting_Classical (x: Bool[], jasmine: Bool[]) : Bool {
        for i in IndexRange(x) {
            if ((not x[i]) and (not jasmine[i])) {
                // They have a day that they can both meet
                return true;
            }
        }
        
        // At least one of them is busy every day of the week
        return false;
    }

    operation Test_Meeting_Oracle () : Unit {
        // There are 2^5 ways to arrange each of the schedules - let's try all of them
        for k in 0..((2^5)-1) { 
            for j in 0..((2^5)-1) {
                // Convert your and Jasmine's schedules to bit arrays
                let binaryX = IntAsBoolArray(k, 5);
                let binaryJasmine = IntAsBoolArray(j, 5);
                
                // Calculate the function classically
                let classicalResult = Meeting_Classical(binaryX, binaryJasmine);
                    
                // create a register of qubits so we can represent
                // your schedule, jasmine's schedule, and the output
                use (x, jasmine, target) = (Qubit[5], Qubit[5], Qubit());
                // Prepare the quantum schedules in basis states matching the binary schedules
                ApplyPauliFromBitString(PauliX, true, binaryX, x);
                ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);

                // Apply the quantum oracle
                Meeting_Oracle(x, jasmine, target);

                // Check that the result of the quantum algorithm matched that
                // of the classical algorithm
                AssertQubit(classicalResult ? One | Zero, target);

                // Undo the preparation of basis states x and jasmine
                ApplyPauliFromBitString(PauliX, true, binaryX, x);
                ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);

                // Check that the oracle did not change its input states
                AssertAllZero(x);
                AssertAllZero(jasmine);

                Reset(target);
            }
        }
        
        Message("Success!");
    }

}
