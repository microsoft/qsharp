namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (V, edges) in Most(ExampleGraphs()) {
            if not CheckOracleRecognizesColoring(V, edges, Kata.Oracle_WeakColoring, IsWeakColoringValid_Reference) {
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
