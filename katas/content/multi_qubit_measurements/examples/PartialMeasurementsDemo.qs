namespace Kata {
    import Std.Diagnostics.*;
    import Std.Convert.*;
    import Std.Math.*;

    @EntryPoint()
    operation DemoPartialMeasurement() : Unit {
        let expected_probabilities = [0.833, 0.167];

        let numRuns = 1000;
        mutable countArray = [0, 0];
        use qs = Qubit[2];
        for i in 1..numRuns {
            // Prepare the Hardy state |𝜓❭ = (1/√12)(3|00⟩ + |01⟩ + |10⟩ + |11⟩)
            Ry(2. * ArcCos(Sqrt(5. / 6.)), qs[0]);
            ApplyControlledOnInt(0, Ry, [qs[0]], (2. * ArcCos(3. / Sqrt(10.)), qs[1]));
            Controlled H([qs[0]], qs[1]);

            if i == 1 {
                Message("The state |𝜓❭ of the system before measurement is:");
                DumpMachine();
            }

            // Measure the first qubit.
            let outcome = M(qs[0]) == Zero ? 0 | 1;
            countArray[outcome] += 1;

            if countArray[outcome] == 1 {
                // The first time the outcome is 0/1, print the system state afterwards.
                Message($"For outcome {outcome}, the post-measurement state of the system is:");
                DumpMachine();
            }
            ResetAll(qs);
        }

        // Obtain simulated probability of measurement for each outcome.
        mutable simulated_probabilities = [];
        for i in 0..1 {
            set simulated_probabilities += [IntAsDouble(countArray[i]) / IntAsDouble(numRuns)];
        }

        Message($"Theoretical measurement probabilities are {expected_probabilities}");
        Message($"Simulated measurement probabilities are {simulated_probabilities}");
    }
}
