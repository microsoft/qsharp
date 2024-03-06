namespace Kata { 
    open Microsoft.Quantum.Math;

    operation ComplexExpReal(r : Double, x : Complex) : Complex {
        if r == 0.0 {return Complex(0.0, 0.0);}
        let a = x::Real;
        let b = x::Imag;
 
        let ra = r^a;
        let lnr = Log(r);
       
        let real = ra * Cos(b * lnr);
        let imaginary = ra * Sin(b * lnr);
 
        return Complex(real, imaginary);
        }
}
