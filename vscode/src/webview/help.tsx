// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function HelpPage() {
  return (
    <div class="qs-help">
      <h1>Azure Quantum Development Kit</h1>
      <h2>Overview</h2>
      <p>
        Welcome to the Azure Quantum Development Kit. Below you will find a
        quick overview of some of the features it enables in Visual Studio Code.
      </p>
      <h2>Powerful Q# editing</h2>
      <p>
        The Q# language is designed for maximum expressivity and productivity
        when writing quantum code. Its type system allows for rich editor
        features such as completion lists, signature help, go to definition,
        find all references, renaming of functions and variables, and more.
      </p>
      <h2>Quantum simulation</h2>
      <p>
        The built-in quantum simulator enables you to run your Q# code directly
        in VS Code. Use the 'Play' icon at the top right of the editor, (or the
        Ctrl+F5 keyboard shortcut), and the output from the simulator will
        appear in the Debug Console of Visual Studio Code. Use the "Q#: Show
        histogram" command to run a number of shots and display the results as a
        histogram.
      </p>
      <h2>Debugging Q# code</h2>
      <p>
        The quantum simulator also supports debugging. Using the 'Play' icon
        drop-down at the top right of the editor select "Debug Q# file", (or use
        the F5 keyboard shortcut). You can set breakpoint and step through code,
        viewing both the classical and quantum state.
      </p>
      <h2>Run jobs on real quantum hardware</h2>
      <p>
        If you have an Azure subscription, you can connect to your Azure Quantum
        workspace and run your Q# programs directly on real quantum machines
        using the 'Quantum Workspaces' view in the Explorer sidebar.
      </p>
      <h2>Learn more</h2>
      <p>
        You can find out more about the latest features in the Azure Quantum
        Development Kit by visiting the&nbsp;
        <a href="https://github.com/microsoft/qsharp/wiki/Editing-Q%23-in-VS-Code">
          wiki on our open source GitHub repository
        </a>
        .&nbsp;Happy coding!
      </p>
    </div>
  );
}
