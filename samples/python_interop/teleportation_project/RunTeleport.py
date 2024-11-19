import qsharp

# set the root folder for the Q# project
# the root folder of a Q# project is where the qsharp.json file is located
# make adjustments to the path depending on the location of the qsharp.json file

# this example assumes your Python program is in the same folder as the qsharp.json file
qsharp.init(project_root=".")

print(qsharp.eval("RunTeleport.Main()"))
