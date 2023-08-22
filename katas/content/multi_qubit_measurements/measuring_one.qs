namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation DemoBasisMeasurement() : Unit {
        let numRuns = 1000;

        // Define coefficients and obtain measurement probabilities for the
        // state |psi❭ = 0.33 |00❭ + 0.67 |01❭ + 0.67 |11❭
        // Use little endian format to encode basis states as integer indices.
        let coefficients = [0.333, 0.0, 0.667, 0.667];
        let expected_probabilities = [0.111, 0.0, 0.445, 0.445];

        // Set up counter array for measurements.
        mutable countArray = [0, 0, 0, 0];

        use qs = Qubit[2];
        for i in 1 .. numRuns {
            // Prepare the state from Exercise 1:
            PrepareEx1State(qs);
            if i == 1 {
                Message("The state |psi❭ of the system before measurement is:");
                DumpMachine();
            }

            // Measure the first qubit, followed by the second qubit, and
            // convert the result to little endian integer
            let result = MeasureInteger(qs);

            // Update countArray
            set countArray w/= result <- countArray[result] + 1;
        }

        // Obtain simulated probability of measurement for each outcome
        mutable simulated_probabilities = [];
        for i in 0 .. 3 {
            set simulated_probabilities +=
                [IntAsDouble(countArray[i]) / IntAsDouble(numRuns)];
        }

        Message($"Theoretical measurement probabilities are {expected_probabilities}");
        Message($"Simulated measurement probabilities are {simulated_probabilities}");
    }

    operation PrepareEx1State(q: Qubit[]): Unit {
        Ry(-ArcCos(1.0/9.0), q[0]);
        within {
            S(q[1]);
            H(q[1]);
        } apply {
            Rz(ArcTan(0.5), q[1]);
            CNOT(q[0], q[1]);
            Rz(ArcTan(0.5)-PI(), q[1]);
            CNOT(q[0], q[1]);
        }
    }
}