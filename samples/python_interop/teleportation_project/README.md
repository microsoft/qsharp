# An example multi-file Q# project with Python classical host

The example project demonstrates a Q# quantum teleportation program implemented as a multi-file project, designed to be executed from Python classical code. 

The `qsharp.json` manifest is located at the root folder. The `src` directory includes the Q# source files which are organized into subfolders. The main file, `RunTeleport.qs`, contains the entry point and references operations defined in other files. The teleportation logic is implemented in `Teleport.qs`, which uses a standard operation from `PrepareState.qs` to create a Bell pair.

You can execute this project by navigating to its root folder and running `python .\RunTeleport.py`. Alternatively, you can open the folder in Visual Studio Code, open the file `RunTeleport.qs`, and select **Run**.

Full details are available on the [Microsoft Learn page](https://learn.microsoft.com/azure/quantum/user-guide/how-to-work-with-qsharp-projects?tabs=tabid-qsharp%2Ctabid-python-run#example-project).
