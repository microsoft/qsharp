namespace Particle {
    struct Particle { x : Int, y : Int, z : Int }

    function addParticles(a : Particle, b : Particle) : Particle {
        return Particle(a.x + b.x, a.y + b.y, a.z + b.z);
    }
}
