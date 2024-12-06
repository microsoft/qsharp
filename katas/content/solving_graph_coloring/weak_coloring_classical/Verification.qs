namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let testGraphs = ExampleGraphs()[0..4];
        let testColorings = [
            // Every coloring would pass on a disconnected graph of 3 vertices
            [([0, 0, 0], true), ([2, 1, 3], true)],
            // Every coloring  would pass on a fully connected graph of 4 vertices,
            // except for the last coloring in which all vertices are of the same color.
            [([0, 2, 1, 3], true), ([3, 0, 1, 0], true), ([0, 0, 0, 0], false)],
            // The colorings for 5-vertex graphs:
            // - the first one is invalid for all graphs except disconnected
            // - the second one is valid for all types of graphs regardless of their structure
            // - two colorings that is valid or invalid depending on the graph
            [([0, 0, 0, 0, 0], false), ([0, 1, 2, 3, 4], true), ([0, 1, 1, 2, 0], false), ([0, 0, 1, 1, 1], true)],
            [([0, 0, 0, 0, 0], false), ([0, 1, 2, 3, 4], true), ([0, 1, 1, 2, 0], true), ([0, 0, 1, 1, 1], false)],
            [([0, 0, 0, 0, 0], false), ([0, 1, 2, 3, 4], true), ([0, 1, 1, 2, 0], true), ([0, 0, 1, 1, 1], true)]
        ];
        for ((V, edges), colorings) in Zipped(testGraphs, testColorings) {
            for (coloring, expected) in colorings {
                if Kata.IsWeakColoringValid(V, edges, coloring) != expected {
                    Message($"Weak coloring {coloring} evaluated incorrectly for graph V = {V}, edges = {edges}: expected {expected}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
