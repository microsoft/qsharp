include "stdgates.inc";



qubit q;
reset q;
x q;
h q;
bit c = measure q;
