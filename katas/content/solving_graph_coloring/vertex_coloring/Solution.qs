namespace Kata {
    operation Oracle_VertexColoring(V : Int, edges: (Int, Int)[], x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let edgesNumber = Length(edges);
        use conflicts = Qubit[edgesNumber];
        within {
            for i in 0 .. edgesNumber - 1 {
                let (v0, v1) = edges[i];
                Oracle_ColorEquality(x[2 * v0 .. 2 * v0 + 1], 
                                     x[2 * v1 .. 2 * v1 + 1], conflicts[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, conflicts, y);
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
