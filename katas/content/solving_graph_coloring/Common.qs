namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import Std.Diagnostics.*;
    import KatasUtils.*;

    // Hardcoded graphs used for testing the vertex coloring problem:
    //  - trivial graph with zero edges
    //  - complete graph with 4 vertices (4-colorable)
    //  - disconnected graph
    //  - random connected graph with more edges and vertices (3-colorable)
    //  - regular-ish graph with 5 vertices (3-colorable, as shown at https://en.wikipedia.org/wiki/File:3-coloringEx.svg without one vertex)
    //  - 6-vertex graph from https://en.wikipedia.org/wiki/File:3-coloringEx.svg
    function ExampleGraphs() : (Int, (Int, Int)[])[] {
        return [
            (3, []),
            (4, [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]),
            (5, [(4, 0), (2, 1), (3, 1), (3, 2)]),
            (5, [(0, 1), (1, 2), (1, 3), (3, 2), (4, 2), (3, 4)]),
            (5, [(0, 1), (0, 2), (0, 4), (1, 2), (1, 3), (2, 3), (2, 4), (3, 4)]),
            (6, [(0, 1), (0, 2), (0, 4), (0, 5), (1, 2), (1, 3), (1, 5), (2, 3), (2, 4), (3, 4), (3, 5), (4, 5)])
        ];
        // Graphs with 6+ vertices can take several minutes to be processed;
        // in the interest of keeping test runtime reasonable we're limiting most of the testing to graphs with 5 vertices or fewer.
    }


    function IsVertexColoringValid_Reference(V : Int, edges : (Int, Int)[], colors : Int[]) : Bool {
        for (start, end) in edges {
            if colors[start] == colors[end] {
                return false;
            }
        }
        return true;
    }


    operation ReadColoring_Reference(nBits : Int, qs : Qubit[]) : Int[] {
        let colorPartitions = Chunks(nBits, qs);
        let measureColor = qs => ResultArrayAsInt(Reversed(MeasureEachZ(qs)));
        return ForEach(measureColor, colorPartitions);
    }


    // Helper function specific to Graph Coloring kata.
    operation CheckOracleRecognizesColoring(
        V : Int,
        edges : (Int, Int)[],
        oracle : (Int, (Int, Int)[], Qubit[], Qubit) => Unit,
        classicalFunction : (Int, (Int, Int)[], Int[]) -> Bool
    ) : Bool {
        // Message($"Testing V = {V}, edges = {edges}");
        let N = 2 * V;
        use (coloringRegister, target) = (Qubit[N], Qubit());
        // Try all possible colorings of 4 colors on V vertices and check if they are calculated correctly.
        // Hack: fix the color of the first vertex, since all colorings are agnostic to the specific colors used.
        for k in 0..(1 <<< (N - 2)) - 1 {
            // Prepare k-th coloring
            let binary = [false, false] + IntAsBoolArray(k, N - 2);
            ApplyPauliFromBitString(PauliX, true, binary, coloringRegister);

            // Read out the coloring (convert one bitmask into V integers) - does not change the state
            let coloring = ReadColoring_Reference(2, coloringRegister);

            // Apply the oracle
            oracle(V, edges, coloringRegister, target);

            // Check that the oracle result matches the classical result
            let val = classicalFunction(V, edges, coloring);
            if val {
                X(target);
            }
            // Uncompute
            ApplyPauliFromBitString(PauliX, true, binary, coloringRegister);

            if not CheckAllZero(coloringRegister + [target]) {
                Message($"Incorrect result for V = {V}, edges = {edges}");
                Message($"For test case with input = {BoolArrayAsKetState(binary)}, coloring = {coloring}");
                if not CheckZero(target) {
                    Message($"Expected answer = {val}, got {not val}");
                } else {
                    Message("The input state should not change");
                }
                ResetAll(coloringRegister + [target]);
                return false;
            }
        }

        true
    }

    function IsWeakColoringValid_OneVertex_Reference(V : Int, edges : (Int, Int)[], colors : Int[], vertex : Int) : Bool {
        mutable neighborCount = 0;
        mutable hasDifferentNeighbor = false;

        for (start, end) in edges {
            if start == vertex or end == vertex {
                set neighborCount += 1;
                if colors[start] != colors[end] {
                    set hasDifferentNeighbor = true;
                }
            }
        }

        return neighborCount == 0 or hasDifferentNeighbor;
    }

    function IsWeakColoringValid_Reference(V : Int, edges : (Int, Int)[], colors : Int[]) : Bool {
        for v in 0..V - 1 {
            if not IsWeakColoringValid_OneVertex_Reference(V, edges, colors, v) {
                return false;
            }
        }

        return true;
    }

}
