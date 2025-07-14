include "stdgates.inc";



qubit q;
reset q;
x q;
h q;
bit c = measure q;

def f() {
    int b = 3;
    int c = 4;
    int d = 5;
}

def g() {
    int a = 2;
    f();
}

g();
