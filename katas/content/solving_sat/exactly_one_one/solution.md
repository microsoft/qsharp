Consider the set of all bit strings $x$ of length $n$ which have only one bit of $x$ equal to $1$.   
This set of bit strings is $S=\{00...01, 00...10, ..., 00..1..00, ..., 01...00, 10...00\}$, 
or, if we convert the bit strings to integers, 
$$S=\{1,2,4,..2^i,..,2^{n-1}\} = \{2^k: 0 \le k \le n-1\}$$

You need to implement an oracle that flips $\ket{y}$ if the input basis state $\ket{x}$ corresponds to one of the bit strings in $S$.
The easiest way to do this is to use $n$ controlled $X$ gates, with `x` as control and the qubit `y` flipped if and only if the control register is in a particular state of the form $2^k$.
You can use the library operation `ApplyControlledOnInt` for this with integers $1, 2, 4, ...$ as controls.
Notice that this operation uses little endian notation to convert integers to bit strings, but it doesn't matter for this exercise.
You need to iterate through all bit strings that have exactly one $1$ in their notation, and the order in which you iterate is not important.

@[solution]({
    "id": "solving_sat__exactly_one_one_solution",
    "codePath": "./Solution.qs"
})
