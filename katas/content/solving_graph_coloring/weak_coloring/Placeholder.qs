namespace Kata {
    import Std.Arrays.*;

    operation Oracle_WeakColoring(
        V : Int,
        edges : (Int, Int)[],
        x : Qubit[],
        y : Qubit
    ) : Unit is Adj + Ctl {
        // Implement your solution here...

    }

    // You might find these helper operations from earlier tasks useful.
    operation Oracle_WeakColoring_OneVertex(
        V : Int,
        edges : (Int, Int)[],
        x : Qubit[],
        y : Qubit,
        vertex : Int
    ) : Unit is Adj + Ctl {
        let neighborEdges = Filtered((a, b) -> a == vertex or b == vertex, edges);
        let nNeighbors = Length(neighborEdges);
        use sameColorChecks = Qubit[nNeighbors];
        within {
            for ((a, b), checkQubit) in Zipped(neighborEdges, sameColorChecks) {
                Oracle_ColorEquality(
                    x[a * 2 .. a * 2 + 1],
                    x[b * 2 .. b * 2 + 1],
                    checkQubit
                );
            }
        } apply {
            X(y);
            if nNeighbors > 0 {
                Controlled X(sameColorChecks, y);
            }
        }
    }

    operation Oracle_ColorEquality(x0 : Qubit[], x1 : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        within {
            for i in 0..Length(x0) - 1 {
                CNOT(x0[i], x1[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, x1, y);
        }
    }
}
