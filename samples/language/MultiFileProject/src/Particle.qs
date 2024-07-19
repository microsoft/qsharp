namespace Particle {
    export Particle;
    newtype Particles = (x : Int, y : Int, z : Int);

    function addParticles(a : Particles, b : Particles) : Particles {
        let (x1, y1, z1) = a!;
        let (x2, y2, z2) = b!;
        return Particles(x1 + x2, y1 + y2, z1 + z2);
    }
}
