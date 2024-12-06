namespace Kata {
    import Std.Math.*;

    function ComplexToComplexPolar(x : Complex) : ComplexPolar {
        let (a, b) = (x.Real, x.Imag);
        return ComplexPolar(Sqrt(a * a + b * b), ArcTan2(b, a));
    }
}
