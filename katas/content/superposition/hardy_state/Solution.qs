namespace Kata {
    open Microsoft.Quantum.Math;

    operation Hardy_State (qs : Qubit[]) : Unit is Adj {

        // Rotate first qubit to (Sqrt(10.0/12.0) |0⟩ + Sqrt(2.0/12.0) |1⟩)
        let theta = ArcCos(Sqrt(10.0/12.0));
        Ry(2.0 * theta, qs[0]);

        ApplyControlledOnInt(0, Ry, [qs[0]], (2.0 * ArcCos(3.0/Sqrt(10.0)), qs[1]));
        ApplyControlledOnInt(1, Ry, [qs[0]], (2.0 * PI()/4.0, qs[1]));
    }
}
