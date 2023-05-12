# Supported Language Features

This document captures the current status of supported and unsupported language features for the compiler
and evaluator. Where possible, alternatives are provided with examples. This is a living document reflecting
current status rather than a proposal of planned support, so it will be updated and changed to reflect
the current status.

***Note on Expression Syntax***

Q# has been moved to an expression based syntax, rather than a statement based syntax. This allows for
new combinations of syntax to be valid that previously were not supported. These are purely additive
and are not called out specifically here.

## Unsupported Features

### User Defined Types, Field Accessors, and the Unwrap Operator

```qsharp
newtype Complex = (Real: Double, Imaginary: Double);
let compl = Complex(1.0, 0.0);
let real = compl::Real;
let (real, imag) = compl!;
```

### Partial Application

```qsharp
let f = R(PauliY, angle, _);
```

### Lambdas

```qsharp
let funcLambda = x -> x + 1;
let opLambda = q => H(q);
```

## Supported Features

- Literals for `Int`, `BigInt`, `Double`, `Bool`, `Pauli`, `Result`, and `String`
- Array declaration with either explicit (`[1, 2, 3]`) or array repeat (`[Zero, size = 4]`) syntax
- Array indexing
- Array copy-update (`arr w/ index <- val`)
- Locally bound immutable (`let x = 4;`) or mutable (`mutable x = 4;`) variables
- Updates to mutable variables via set-expressions
- Concatenation of strings and arrays
- Arithmetic operations, including assignment variations (ie: `x + 1` and `set x += 1`)
- User failure expressions (`fail "This failed";`)
- For-loops, while-loops, and repeat-until-success loops
- Conditional control flow in if-expressions and conditional ternary expressions (`cond ? thenVal | elseVal`)
- Invoking of callables
- Return expressions
- Functor application for `Adjoint` and `Controlled` with nesting
- Qubit use- and borrow-statements
- Explicitly declared specializations (ie: `body`, `adjoint`, `controlled`, and `controlled adjoint`)
- Self-adjoint generator (`adjoint self`)
- Body intrinsic callables (`body intrinsic`) (*Limitation:* only specific intrinsic callables are
supported by the evaluator, matching the callables present in the standard library)
- Callables as arguments to other callables (ie: `operation ApplyToEach(op : (Qubit => Unit), q : Qubit) : Unit {}`)
- Controlled generator (`controlled distribute` and `controlled adjoint distribute`)
- Adjoint generator (`adjoint invert` and `controlled adjoint invert`)
- Conjugate expressions (within-apply)
- String interpolation
