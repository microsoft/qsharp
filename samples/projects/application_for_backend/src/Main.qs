import Std;
import ContosoBackend;

operation Main() : Result[] {
    use qs = Qubit[4];
    GateSet.Rx(Math.PI(), qs[0]);
    GateSet.Rz(Math.PI(), qs[1]);
    GateSet.Rzz(Math.PI(), qs[2], qs[3]);

    [
        GateSet.MResetZ(qs[0]),
        GateSet.MResetZ(qs[1]),
        GateSet.MResetZ(qs[2]),
        GateSet.MResetZ(qs[3])
    ]
}
