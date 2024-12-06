namespace Kata {
    import Std.Math.*;

    function ComplexExponent(x : Complex) : Complex {
        Complex(E()^x.Real * Cos(x.Imag), E()^x.Real * Sin(x.Imag))
    }
}
