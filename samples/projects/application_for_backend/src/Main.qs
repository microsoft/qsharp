/// # Sample
/// Quantum Backend Application
///
/// # Description
/// This Q# file contains a program developed for the hypothetical Contoso
/// quantum backend. It uses the native gateset for the backend provided by the
/// "ContosoBackend" dependency.
import Std.Math.PI;
import ContosoBackend.*;

operation Main() : Result[] {
    use qs = Qubit[4];

    // Use the native gateset to manipulate and measure qubits.
    GateSet.Rx(PI(), qs[0]);
    GateSet.Rz(PI(), qs[1]);
    GateSet.Rzz(PI(), qs[2], qs[3]);
    return [
        GateSet.MResetZ(qs[0]),
        GateSet.MResetZ(qs[1]),
        GateSet.MResetZ(qs[2]),
        GateSet.MResetZ(qs[3])
    ];
}
