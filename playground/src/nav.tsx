// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { samples } from "./samples.js";

const katas = [
    "1. Single qubit gates",
    "2. Multi qubit gates"
];

export function Nav(props: {sampleSelected: (name: string) => void}) {
    return (
      <nav class="nav-column">
        <div class="nav-1 nav-selectable">Editor</div>

        <div class="nav-1">Samples</div>

        {Object.keys(samples).map(name => (
          <div class="nav-2 nav-selectable" onClick={() => props.sampleSelected(name)}>{name}</div>
        ))}

        <div class="nav-1">Katas</div>
        {katas.map(name => (<div class="nav-2 nav-selectable">{name}</div>))}
      </nav>);
}
