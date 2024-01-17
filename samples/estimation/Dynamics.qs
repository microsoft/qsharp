/// # Sample
/// Quantum Dynamics
///
/// # Description
/// This example demonstrates quantum dynamics in a style tailed for
/// resource estimation. The sample is specifically the simulation
/// of an Ising model Hamiltonian on an NxN 2D lattice using a
/// fourth-order Trotter Suzuki product formula, assuming
/// a 2D qubit architecture with nearest-neighbor connectivity.
/// The is an example of a program that is not amenable to simulating
/// classically, but can be run through resource estimation to determine
/// what size of quantum system would be needed to solve the problem.
namespace QuantumDynamics {
    open Microsoft.Quantum.Math;

    @EntryPoint()
    operation Main() : Unit {
        let N = 10;
        let J = 1.0;
        let g = 1.0;
        let totTime = 20.0;
        let dt = 0.25;
        let eps = 0.010;

        IsingModel2DSim(N, J, g, totTime, dt, eps);
    }

    function GetQubitIndex(row : Int, col : Int, n : Int) : Int {
        return row % 2 == 0             // if row is even,
            ? col + n * row             // move from left to right,
            | (n - 1 - col) + n * row;  // otherwise from right to left.
    }

    function SetSequences(len : Int, p : Double, dt : Double, J : Double, g : Double) : (Double[], Double[]) {
        // create two arrays of size `len`
        mutable seqA = [0.0, size=len];
        mutable seqB = [0.0, size=len];

        // pre-compute values according to exponents
        let values = [
            -J * p * dt,
            g * p * dt,
            -J * p * dt,
            g * p * dt,
            -J * (1.0 - 3.0 * p) * dt / 2.0,
            g * (1.0 - 4.0 * p) * dt,
            -J * (1.0 - 3.0 * p) * dt / 2.0,
            g * p * dt,
            -J * p * dt,
            g * p * dt
        ];

        // assign first and last value of `seqA`
        set seqA w/= 0 <- -J * p * dt / 2.0;
        set seqA w/= len - 1 <- -J * p * dt / 2.0;

        // assign other values to `seqA` or `seqB`
        // in an alternating way
        for i in 1..len - 2 {
            if i % 2 == 0 {
                set seqA w/= i <- values[i % 10];
            }
            else {
                set seqB w/= i <- values[i % 10];
            }
        }

        return (seqA, seqB);
    }

    operation ApplyAllX(qs : Qubit[], theta : Double) : Unit {
        // This applies `Rx` with an angle of `2.0 * theta` to all qubits in `qs`
        // using partial application
        ApplyToEach(Rx(2.0 * theta, _), qs);
    }

    operation ApplyDoubleZ(n : Int, qs : Qubit[], theta : Double, dir : Bool, grp : Bool) : Unit {
        let start = grp ? 0 | 1;    // Choose either odd or even indices based on group number

        for i in 0..n - 1 {
            for j in start..2..n - 2 {    // Iterate through even or odd `j`s based on `grp`
                // rows and cols are interchanged depending on direction
                let (row, col) = dir ? (i, j) | (j, i);

                // Choose first qubit based on row and col
                let ind1 = GetQubitIndex(row, col, n);
                // Choose second qubit in column if direction is horizontal and next qubit in row if direction is vertical
                let ind2 = dir ? GetQubitIndex(row, col + 1, n) | GetQubitIndex(row + 1, col, n);

                within {
                    CNOT(qs[ind1], qs[ind2]);
                } apply {
                    Rz(2.0 * theta, qs[ind2]);
                }
            }
        }
    }

    operation IsingModel2DSim(N : Int, J : Double, g : Double, totTime : Double, dt : Double, eps : Double) : Unit {
        use qs = Qubit[N * N];
        let len = Length(qs);

        let p = 1.0 / (4.0 - (4.0 ^ (1.0 / 3.0)));
        let t = Ceiling(totTime / dt);

        let seqLen = 10 * t + 1;

        let (seqA, seqB) = SetSequences(seqLen, p, dt, J, g);

        for i in 0..seqLen - 1 {
            // for even indexes
            if i % 2 == 0 {
                ApplyAllX(qs, seqA[i]);
            } else {
                // iterate through all possible combinations for `dir` and `grp`.
                for (dir, grp) in [(true, true), (true, false), (false, true), (false, false)] {
                    ApplyDoubleZ(N, qs, seqB[i], dir, grp);
                }
            }
        }
    }
}
