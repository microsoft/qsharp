import Std;
import ContosoBackend;

operation Main() : (Result, Result, Result[]) {
    use rxQubit = Qubit();
    GateSet.Rx(Math.PI(), rxQubit);

    use rzQubit = Qubit();
    GateSet.Rz(Math.PI(), rzQubit);

    use rzzRegister = Qubit[2];
    GateSet.Rzz(Math.PI(), rzzRegister[0], rzzRegister[1]);

    let output = (GateSet.Mz(rxQubit), GateSet.Mz(rzQubit), [GateSet.Mz(rzzRegister[0]), GateSet.Mz(rzzRegister[1])]);

    GateSet.Reset(rxQubit);
    GateSet.Reset(rzQubit);
    for q in rzzRegister {
        GateSet.Reset(q);
    }
    output
}
