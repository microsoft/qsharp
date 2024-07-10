import qsharp

qsharp.init(project_root=".")

max = input("Enter the max number: ")
(results, number) = qsharp.eval(f"GenerateRandom.GenerateRandomNumbers(0, {max})")

count = 0
for result in results:
    if result == qsharp.Result.One:
        count += 1

print(f"Number of Ones: {count}")
print(f"The random number generated between 0 to {max}: {number}")
print(results)
