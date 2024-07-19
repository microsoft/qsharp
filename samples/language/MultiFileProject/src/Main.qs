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
    import Particle.*;
    @EntryPoint()
    function Main() : Unit {
        let particleA = Particles(0, 0, 0);
        let particleB = Particles(1, 1, 1);

        let particleC = addParticles(particleA, particleB);
        Message($"Particle C: Particle({particleC::x}, {particleC::y}, {particleC::z})")
    }
}
