import qsharp

qsharp.init(project_root=".")

nQubits = input("Enter the number of random bits to be generated: ")
(results, number) = qsharp.eval(
    f"GenerateRandomNumbers.GenerateRandomNumbers({nQubits})"
)

count = 0
for result in results:
    if result == qsharp.Result.One:
        count += 1

print(f"Bits generated: {results}")
print(f"Number of Ones: {count}")
print(f"The integer representation of the generated {nQubits} bits: {number}")
