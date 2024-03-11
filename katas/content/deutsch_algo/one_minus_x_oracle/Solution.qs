namespace Kata {
    open Microsoft.Quantum.Math;
    
    operation PhaseOracle_OneMinusX(x : Qubit) : Unit is Adj + Ctl {
        Z(x);
        R(PauliI, 2.0 * PI(), x);
    }
}
