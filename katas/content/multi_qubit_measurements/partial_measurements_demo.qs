namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation DemoPartialMeasurement() : Unit {
        let numRuns = 1000;
        let divider = "--------------------------------------------------------------------------------------------------";
        // 
        // We can use coefficients without normalization in PrepareArbitraryStateD, 
        // the operation will normalize them automatically.
        let coefficients = [3., 1., 1., 1.]; 
        let expected_probabilities = [0.833, 0.167];
        
        // Set up the counter array for measurements.
        mutable countArray = [0, 0];
        
        use qs = Qubit[2];
        for i in 1 .. numRuns {
            // Prepare the state from Exercise 4:
            // |𝜓❭ = (1/√12)(3|00⟩+|01⟩+|10⟩+|11⟩) 
            PrepareHardyState(qs);
                
            // Display the state of the qubits.
            if i == 1 {
                Message("The state |𝜓❭ of the system before measurement is:");
                DumpMachine();
                Message(divider);
            }

            // Measure the first qubit.
            let outcome = M(qs[0]) == Zero ? 0 | 1;
            set countArray w/= outcome <- countArray[outcome] + 1;
            
            if countArray[outcome] == 1 { 
                // The first time the outcome is 0/1, print the system state afterwards.
                Message("For outcome {outcome}, the post-measurement state of the system is:");
                DumpMachine();
            }
            ResetAll(qs);
        }
        
        // Obtain simulated probability of measurement for each outcome
        mutable simulated_probabilities = [];
        for i in 0 .. 1 {
            set simulated_probabilities += [IntAsDouble(countArray[i]) / IntAsDouble(numRuns)];
        }
        
        Message($"Theoretical measurement probabilities are {expected_probabilities}");
        Message($"Simulated measurement probabilities are {simulated_probabilities}");
    }
    
    operation PrepareHardyState(q: Qubit[]): Unit {
        Ry(ArcCos(2.0/3.0), q[1]);
        within {
            S(q[0]);
            H(q[0]);
        } apply {
            CNOT(q[1], q[0]);
            Rz(ArcTan(1.0/2.0), q[0]);
            CNOT(q[1], q[0]);
            Rz(-ArcTan(2.0), q[0]);
        }        
    }

}
