namespace Kata.Verification {
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import Std.Math.*;
    import KatasUtils.*;

    operation ReconstructMessageWhenThreeEntangledQubitsTestLoop(
        reconstructMessage : (Qubit, (Bool, Bool), Bool) => Unit
    ) : Bool {

        let setupPsiOps = [(I, "|0⟩"), (X, "|1⟩"), (H, "|+⟩"), (Ry(ArcCos(0.6) * 2.0, _), "0.6|0⟩ + 0.8|1⟩")];
        let numRepetitions = 100;

        for (psiOp, psiName) in setupPsiOps {
            for j in 1..numRepetitions {
                use (qMessage, qAlice, qBob, qCharlie) = (Qubit(), Qubit(), Qubit(), Qubit());
                psiOp(qMessage);
                EntangleThreeQubitsWrapper_Reference([qAlice, qBob, qCharlie]);
                let (b1, b2) = SendMessage_Reference(qAlice, qMessage);
                let b3 = ResultAsBool(M(qBob));
                reconstructMessage(qCharlie, (b1, b2), b3);
                Adjoint psiOp(qCharlie);
                if not CheckZero(qCharlie) {
                    Message($"Incorrect. The state {psiName} was teleported incorrectly.");
                    psiOp(qCharlie);
                    Message("The state of the qubits [qMessage, qAlice, qBob, qCharlie] after teleportation:");
                    DumpMachine();
                    ResetAll([qMessage, qAlice, qBob, qCharlie]);
                    return false;
                }
                ResetAll([qMessage, qAlice, qBob, qCharlie]);
            }
        }
        Message($"Correct!");
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return ReconstructMessageWhenThreeEntangledQubitsTestLoop(Kata.ReconstructMessageWhenThreeEntangledQubits);
    }
}
