namespace Kata {
    operation QuantumFourierTransform(j : Qubit[]) : Unit is Adj + Ctl {
        let n = Length(j);
        for ind in 0 .. n - 1 {
            BinaryFractionQuantumInPlace(j[ind ...]);
        }
        for ind in 0 .. n / 2 - 1 {
            SWAP(j[ind], j[n - 1 - ind]);
        }        
    }

    operation BinaryFractionQuantumInPlace(j : Qubit[]) : Unit is Adj + Ctl {
        H(j[0]);
        for ind in 1 .. Length(j) - 1 {
            Controlled R1Frac([j[ind]], (2, ind + 1, j[0]));
        }
    }
}