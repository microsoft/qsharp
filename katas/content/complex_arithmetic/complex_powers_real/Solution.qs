namespace Kata { 
    open Microsoft.Quantum.Math;

    function ComplexExpReal(r : Double, x : Complex) : Complex {
        if AbsD(r) < 1e-9 {
            return Complex(0.0, 0.0);
        }
        
        let (a, b) = x!;        
        let ra = r ^ a;
        let lnr = Log(r);
        return Complex(ra * Cos(b * lnr), ra * Sin(b * lnr));
    }
}
