namespace Kata {
    open Microsoft.Quantum.Math;
    
    operation PowersOfI(n : Int) : Int {
   // If n is divisible by 4
    if (n % 4 == 0) {
        return 1;}
    else{
        return -1;}
 }
 
}
