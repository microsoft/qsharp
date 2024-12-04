namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for theta in [0.04, 0.1, 0.25, 0.31, 0.5, 0.87, 1.05, 1.41, 1.66, 1.75, 2.0, 2.16, 2.22, 2.51, 2.93, 3.0, 3.1] {
            Message($"Testing theta = {theta}...");
            if not DistinguishUnitaries_Framework([Rz(theta, _), Ry(theta, _)], Kata.DistinguishRzFromRy(theta, _), ["Rz", "Ry"], -1) {
                return false;
            }
        }
        return true;
    }
}
