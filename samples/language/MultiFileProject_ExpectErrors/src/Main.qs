/// # Sample
/// Multi File Project
///
/// # Description
/// Organizing code into multiple Q# source files is an important part of
/// writing readable and maintainable code. In this project, we have `Main.qs`,
/// and `Particle.qs`, which defines a new namespace for particle operations.
/// The presence of a Q# manifest file (`qsharp.json`) tells the compiler
/// to include all Q# files under `src/`.
namespace MyQuantumApp {
    open Particle;
    @EntryPoint()
    function Main() : Unit {

        // these are coming from local deps
        DependencyA.MagicFunction();
        DependencyB.MagicFunction();

        // DependencyC contains a circular dependency - this should rightfully
        // cause an error to be shown for the qsharp.json, but the language
        // service should still be functional for this line
        DependencyC.MagicFunction();

        // this is coming from github - minestarks/qsharp-project-template
        GitHub.Diagnostics.DumpMachine_();

        let particleA = Particle(0, 0, 0);
        let particleB = Particle(1, 1, 1);

        let particleC = addParticles(particleA, particleB);
    }
}
