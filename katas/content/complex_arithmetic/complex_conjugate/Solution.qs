namespace Kata {
    import Std.Math.*;

    function ComplexConjugate(x : Complex) : Complex {
        Complex(x.Real, -x.Imag)
    }
}
