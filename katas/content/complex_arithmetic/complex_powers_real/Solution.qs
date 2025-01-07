namespace Kata {
    import Std.Math.*;

    function ComplexExpReal(r : Double, x : Complex) : Complex {
        if AbsD(r) < 1e-9 {
            return Complex(0., 0.);
        }

        let ra = r^x.Real;
        let lnr = Log(r);
        return Complex(ra * Cos(x.Imag * lnr), ra * Sin(x.Imag * lnr));
    }
}
