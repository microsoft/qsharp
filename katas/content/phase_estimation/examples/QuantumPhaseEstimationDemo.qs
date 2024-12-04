namespace Kata {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;

    @EntryPoint()
    operation QuantumPhaseEstimationDemo() : Unit {
        // Experiment with the parameters to explore algorithm behavior for different eigenphases!
        // Use R1Frac(k, n, _) for eigenvalue exp(2π𝑖 k/2^(n+1)), eigenphase k/2^(n+1)
        // or R1(theta, _) for eigenvalue exp(2π𝑖 theta/2), eigenphase theta/2
        // Here are some convenient unitaries and their eigenphases
        // R1Frac(1, 0, _)   |  0.5
        // R1Frac(1, 1, _)   |  0.25
        // R1Frac(1, 2, _)   |  0.125
        // R1Frac(1, 3, _)   |  0.0625
        let U = R1Frac(1, 3, _);
        let P = X;     // |1⟩ basis state is convenient to experiment with R1 and R1Frac gates
        let n = 3;
        mutable counts = [0, size = 2^n];
        for _ in 1..100 {
            let res = PhaseEstimation(U, P, n);
            set counts w/= res <- counts[res] + 1;
        }
        for i in 0..2^n - 1 {
            if counts[i] > 0 {
                Message($"Eigenphase {IntAsDouble(i) / IntAsDouble(2^n)} - {counts[i]}%");
            }
        }
    }

    operation PhaseEstimation(
        U : Qubit => Unit is Ctl,
        P : Qubit => Unit,
        n : Int
    ) : Int {
        use (phaseRegister, eigenstate) = (Qubit[n], Qubit());
        P(eigenstate);
        ApplyToEach(H, phaseRegister);
        for k in 0..n - 1 {
            for _ in 1..1 <<< k {
                Controlled U([phaseRegister[k]], eigenstate);
            }
        }
        SwapReverseRegister(phaseRegister);
        Adjoint ApplyQFT(phaseRegister);

        Reset(eigenstate);
        return MeasureInteger(phaseRegister);
    }
}
