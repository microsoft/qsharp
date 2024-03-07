**Input:** The "black box" oracle the implements $f(x)$, defined as an operation that takes a single qubit as an argument and returns `Unit`.

**Goal:** Return `true` if the function is constant ($f(0) = f(1)$), or `false` if it is variable ($f(0) \neq f(1)$).
You can use only one oracle call!
