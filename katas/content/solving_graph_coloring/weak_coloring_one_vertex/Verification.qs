namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (V, edges) in Most(ExampleGraphs()) {
            for vertex in 0..V - 1 {
                if not CheckOracleRecognizesColoring(
                    V,
                    edges,
                    Kata.Oracle_WeakColoring_OneVertex(_, _, _, _, vertex),
                    IsWeakColoringValid_OneVertex_Reference(_, _, _, vertex)
                ) {
                    Message($"Testing vertex {vertex}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
