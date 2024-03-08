namespace Kata {    
     open Microsoft.Quantum.Math;
     
    operation ComplexAdd(x : Complex, y: Complex) : Complex {        
        let (a, b) = x!;
        let (c, d) = y!;

        return Complex(a + c, b + d);
    }
}
