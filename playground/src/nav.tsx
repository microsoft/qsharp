// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const samples = [
    "Teleportation",
    "Random numbers",
    "Deutsch-Jozsa",
    "Grovers search"
];

const katas = [
    "1. Single qubit gates",
    "2. Multi qubit gates"
];

export function Nav(props: any) {
    return (
      <nav class="nav-column">
        <div class="nav-1 nav-selectable">Editor</div>

        <div class="nav-1">Samples</div>
        {samples.map(name => (<div class="nav-2 nav-selectable">{name}</div>))}

        <div class="nav-1">Katas</div>
        {katas.map(name => (<div class="nav-2 nav-selectable">{name}</div>))}
      </nav>);
}
