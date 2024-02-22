namespace Kata.Verification {
   
   operation PowersOfIExp(n : Int) : Int{
   // If n is divisible by 4
    if n % 4 == 0{
        return 1;}
    else{
        return -1;}
 }

@EntryPoint()
operation CheckSolution() : Bool {
mutable i = 1;
mutable n=0;
mutable success = true;
repeat 
{   
   set n = 2*i;
   let expected = PowersOfIExp(n);
   let actual  = Kata.PowersOfI(n);
   if expected != actual {set success = false;}
   set i += 1;   
 }
 until (i > 25) or (success == false);   
 if success == true {Message("Success!");}
else{Message("Solution failed.");}
return success;
}
}
