namespace Kata {    
    open Microsoft.Quantum.Math;
    
    operation ComplexMult(x : Complex, y: Complex) : Complex {

        let (a, b) = x!;
        let (c, d) = y!;

        let real = (a * c) - (b * d);
        let imag = (a * d) + (b * c);
        
        return Complex(real, imag);        
    }
}
