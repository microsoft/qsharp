/// # Sample
/// Minimal Adaptive Quantum Program
///
/// # Description
/// A simple adaptive (aka integrated hybrid) program.
namespace Adaptive {
    open Microsoft.Quantum.Measurement;
    @EntryPoint()
    operation Main() : Result {
        use (q0, q1) = (Qubit(), Qubit());
        H(q0);
        if MResetZ(q0) == One {
            X(q1);
        }
        MResetZ(q1)
    }
}