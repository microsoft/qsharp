namespace Kata.Verification {
    import Std.Diagnostics.*;
    import Std.Math.*;

    operation MeasurementFreeTeleportTestLoop(
        measurementFreeTeleport : (Qubit, Qubit, Qubit) => Unit
    ) : Bool {
        let setupPsiOps = [(I, "|0⟩"), (X, "|1⟩"), (H, "|+⟩"), (Ry(ArcCos(0.6) * 2.0, _), "0.6|0⟩ + 0.8|1⟩")];
        let numRepetitions = 100;

        for (psiOp, psiName) in setupPsiOps {
            for j in 1..numRepetitions {
                use (qMessage, qAlice, qBob) = (Qubit(), Qubit(), Qubit());
                psiOp(qMessage);
                StatePrep_BellState(qAlice, qBob, 0);
                measurementFreeTeleport(qAlice, qBob, qMessage);
                Adjoint psiOp(qBob);
                if not CheckZero(qBob) {
                    Message($"Incorrect. The state {psiName} was teleported incorrectly.");
                    psiOp(qBob);
                    Message("The state of the qubits [qMessage, qAlice, qBob] after teleportation:");
                    DumpMachine();
                    ResetAll([qMessage, qAlice, qBob]);
                    return false;
                }
                ResetAll([qMessage, qAlice, qBob]);
            }
        }

        Message("Correct!");
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return MeasurementFreeTeleportTestLoop(Kata.MeasurementFreeTeleport);
    }
}
