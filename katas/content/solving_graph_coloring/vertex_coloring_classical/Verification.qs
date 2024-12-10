namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let testGraphs = ExampleGraphs()[0..4];
        let testColorings = [
            [([0, 0, 0], true), ([2, 1, 3], true)],
            [([0, 2, 1, 3], true), ([3, 0, 1, 2], true), ([0, 2, 1, 0], false)],
            [([0, 1, 2, 3, 4], true), ([0, 2, 1, 0, 3], true), ([1, 0, 1, 2, 1], false), ([0, 0, 0, 0, 0], false)],
            [([0, 1, 0, 2, 1], true), ([0, 2, 0, 1, 3], true), ([0, 1, 0, 1, 2], false)],
            [([1, 2, 3, 1, 2], true), ([1, 2, 3, 4, 1], false)]
        ];
        for ((V, edges), colorings) in Zipped(testGraphs, testColorings) {
            for (coloring, expected) in colorings {
                if Kata.IsVertexColoringValid(V, edges, coloring) != expected {
                    Message($"Coloring {coloring} evaluated incorrectly for graph V = {V}, edges = {edges}: expected {expected}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
