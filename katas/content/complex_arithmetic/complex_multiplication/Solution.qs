namespace Kata {
    import Std.Math.*;

    function ComplexMult(x : Complex, y : Complex) : Complex {
        let (a, b) = (x.Real, x.Imag);
        let (c, d) = (y.Real, y.Imag);
        return Complex(a * c - b * d, a * d + b * c);
    }
}
