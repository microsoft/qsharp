/// # Sample
/// Multi File Testing Project
///
/// # Description
/// Organizing code into multiple Q# source files is an important part of
/// writing readable and maintainable code. In this project, we have `SWAP.qs`,
/// and `Particle.qs`, which defines a new namespace for particle operations.
/// The presence of a Q# manifest file (`qsharp.json`) tells the compiler
/// to include all Q# files under `src/`.

namespace BellState {
    operation PrepareBellState(qs : Qubit[]) : Unit is Ctl + Adj {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }
}

