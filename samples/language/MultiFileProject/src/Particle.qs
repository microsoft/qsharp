struct Particle { X : Int, Y : Int, Z : Int }

function addParticles(a : Particle, b : Particle) : Particle {
    return new Particle {
        X = a.X + b.X,
        Y = a.Y + b.Y,
        Z = a.Z + b.Z
    };
}
