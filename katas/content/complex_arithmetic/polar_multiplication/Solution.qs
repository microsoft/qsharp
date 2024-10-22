namespace Kata {
    open Microsoft.Quantum.Math;

    function ComplexPolarMult(x : ComplexPolar, y : ComplexPolar) : ComplexPolar {
        mutable theta = x.Argument + y.Argument;
        while theta > PI() {
            set theta -= 2.0 * PI();
        }
        while theta <= -PI() {
            set theta += 2.0 * PI();
        }
        return ComplexPolar(x.Magnitude * y.Magnitude, theta);
    }
}
