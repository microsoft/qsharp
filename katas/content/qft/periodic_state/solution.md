Recall the definition of the QFT: 
for a basis state $\ket{j}$, the QFT is defined as 

$$QFT\ket{j}= \frac1{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i \cdot j k/2^{n}} \ket{k}$$

You can see that using $j = F$ will produce exactly the required state!

To prepare the state $\ket{F}$ on the input register, you need to convert the integer value $F$ into a big endian binary representation and encode it in the register. 

> When writing the code, make sure that you pay attention to whether library functions use little or big endian!  In particular, `IntAsBoolArray` uses little endian encoding, and the QFT operation uses big endian, so the result of converting the integer into a bit string must be reversed before encoding it in the register. 

@[solution]({
    "id": "qft__periodic_state_solution",
    "codePath": "./Solution.qs"
})
