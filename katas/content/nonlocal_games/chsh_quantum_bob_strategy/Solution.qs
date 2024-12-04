namespace Kata {
    import Std.Math.*;

    operation BobQuantum(bit : Bool, qubit : Qubit) : Bool {
        let angle = 2.0 * PI() / 8.0;
        Ry(not bit ? -angle | angle, qubit);
        return M(qubit) == One;
    }
}
