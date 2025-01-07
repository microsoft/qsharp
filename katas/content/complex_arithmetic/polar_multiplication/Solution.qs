namespace Kata {
    import Std.Math.*;

    function ComplexPolarMult(x : ComplexPolar, y : ComplexPolar) : ComplexPolar {
        mutable theta = x.Argument + y.Argument;
        if theta > PI() {
            set theta -= 2.0 * PI();
        }
        if theta <= -PI() {
            set theta += 2.0 * PI();
        }
        return ComplexPolar(x.Magnitude * y.Magnitude, theta);
    }
}
