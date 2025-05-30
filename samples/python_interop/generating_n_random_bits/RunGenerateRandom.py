from pathlib import Path
import qsharp

this_dir = Path(__file__).parent
qsharp.init(project_root=this_dir)

from qsharp.code.GenerateRandomNumbers import GenerateRandomNumbers

nQubits = input("Enter the number of random bits to be generated: ")
(results, number) = GenerateRandomNumbers(int(nQubits))

count = 0
for result in results:
    if result == qsharp.Result.One:
        count += 1

print(f"Bits generated: {results}")
print(f"Number of Ones: {count}")
print(f"The integer representation of the generated {nQubits} bits: {number}")
