import qsharp

# set the root folder for the Q# project
# make adjustments to the path depending on where your program is saved

# this example assumes your program is in the same folder as the root folder
qsharp.init(project_root=".")

print(qsharp.eval("RunTeleport.RunTeleportationExample()"))
