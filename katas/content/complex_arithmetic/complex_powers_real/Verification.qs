namespace Kata.Verification {
    
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;    
    
        operation ComplexExpReal_Reference(r : Double, x : Complex) : Complex {
               
            if r == 0.0 {return Complex(0.0, 0.0);}
            let real = r^x::Real * Cos(x::Imag * Log(r));
            let imaginary = r^x::Real * Sin(x::Imag * Log(r));
            return Complex(real, imaginary);
        }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        for count in 0 .. 24 {
            let testx = ComplexRandom(0., 100.); 
            let testr = DrawRandomDouble(0., 10.); 

            let expected = ComplexExpReal_Reference(testr, testx); 
            let actual = Kata.ComplexExpReal(testr, testx);        
        
            if not(ComplexEqual(expected, actual)) {
                Message($"Incorrect. When x = {testx::Real} + {testx::Imag}i and r = {testr}, actual value doesn't match expected value");
                Message($"Actual value: {actual::Real} + {actual::Imag}i. Expected value: {expected::Real} +  {expected::Imag}i");
                return false;
            }                
        }            

            Message("Correct!");
            return true;        
    }
    
}
