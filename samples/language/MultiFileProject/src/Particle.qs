namespace Particle {
  newtype Particle = (x: Int, y: Int, z: Int);

  function addParticles(a: Particle, b: Particle) : Particle {
    let (x1, y1, z1) = a!;
    let (x2, y2, z2) = b!;
    return Particle(x1 + x2, y1 + y2, z1 + z2);
  }
}
