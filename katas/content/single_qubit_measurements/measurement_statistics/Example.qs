namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation MeasumentStatisticsDemo() : Unit {
        mutable countZero = 0;
        let numRuns = 100;
        use q = Qubit();
        for i in 1 .. numRuns {
            // Prepare the qubit in the superposition state |𝜓❭ = 0.6 |0❭ + 0.8 |1❭
            Ry(2.0 * ArcTan2(0.8, 0.6), q);

            // Measure and update the counts according to the outcomes
            if MResetZ(q) == Zero {
                set countZero += 1;
            }
        }
        let countOne = numRuns - countZero;

        Message($"Simulated probability of measuring 0 is 0.{countZero}.");
        Message($"Theoretical probability of measuring 0 is 0.36.");
        Message($"Simulated probability of measuring 1 is 0.{countOne}.");
        Message($"Theoretical probability of measuring 1 is 0.64.");
    }
}
