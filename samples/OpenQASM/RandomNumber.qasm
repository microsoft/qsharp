OPENQASM 3;
include "stdgates.inc";

def GenerateRandomBit(qubit q) -> bit {
  bit b;

  reset q;
  h q;
  b = measure q;

  return b;
}

def GenerateRandomNumber(qubit q, int nBits) -> int {
  int number = 0;

  for int k in [1:nBits] {
      number |= GenerateRandomBit(q);
      number <<= 1;
  }

  return number;
}

qubit q;
int result;

result = GenerateRandomNumber(q, 5);
