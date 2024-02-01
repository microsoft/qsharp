# Summary
Used to specify that there's only one execution variant in `BeginEstimateCaching`
function
---
function SingleVariant() : Int

---

# Summary
Instructs the resource estimator to stop estimates caching
because the code fragment in consideration is over. This function
is only available when using resource estimator execution target.
---
function EndEstimateCaching() : Unit

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the number of auxilliary qubits is equal to the `amount`.
---
function AuxQubitCount(amount : Int) : (Int, Int)

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the number of the T gates is equal to the `amount`.
---
function TCount(amount : Int) : (Int, Int)

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the number of rotations is equal to the `amount`.
---
function RotationCount(amount : Int) : (Int, Int)

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the rotation depth is equal to the `amount`.
---
function RotationDepth(amount : Int) : (Int, Int)

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the number of the CCZ gates is equal to the `amount`.
---
function CczCount(amount : Int) : (Int, Int)

---

# Summary
Returns a tuple that can be passed to the `AccountForEstimates` operation
to specify that the number Measurements is equal to the `amount`.
---
function MeasurementCount(amount : Int) : (Int, Int)

---

# Summary
Pass the value returned by the function to the `AccountForEstimates` operation
to indicate Parallel Synthesis Sequential Pauli Computation (PSSPC) layout.
See https://arxiv.org/pdf/2211.07629.pdf for details.
---
function PSSPCLayout() : Int

---

# Summary
Account for the resource estimates of an unimplemented operation,
which were obtainted separately. This operation is only available
when using resource estimator execution target.
# Input
## cost
Array of tuples containing resource estimates of the operation. For example,
if the operation uses three T gates, pass the tuple returned by TCount(3)
as one of the array elements.
## layout
Provides the layout scheme that is used to convert logical resource estimates
to physical resource estimates. Only PSSPCLayout() is supported at this time.
## arguments
Operation takes these qubits as its arguments.
---
operation AccountForEstimates(estimates : (Int, Int)[], layout : Int, arguments : Qubit[]) : Unit is Adj

---

# Summary

Instructs the resource estimator to assume that the resources from the
call of this operation until a call to `EndRepeatEstimates` are
accounted for `count` times, without the need to execute the code that many
times. Calls to `BeginRepeatEstimates` and `EndRepeatEstimates` can be nested.
A helper operation `RepeatEstimates` allows to call the two functions in a
`within` block.

# Input
## count
Assumed number of repetitions, factor to multiply the cost with
---
operation BeginRepeatEstimates(count : Int) : Unit is Adj

---

# Summary

Companion operation to `BeginRepeatEstimates`.
---
operation EndRepeatEstimates() : Unit is Adj

---

# Summary

Instructs the resource estimator to assume that the resources from the
call of this operation until a call to `Adjoint RepeatEstimates` are
accounted for `count` times, without the need to execute the code that many
times.

# Input
## count
Assumed number of repetitions, factor to multiply the cost with
---
operation RepeatEstimates(count : Int) : Unit is Adj
