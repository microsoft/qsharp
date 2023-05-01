// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useState } from "preact/hooks";
import { samples } from "./samples.js";

const katas = [
    "1. Single qubit gates",
    "2. Multi qubit gates"
];

export function Nav(props: {sampleSelected: (name: string) => void}) {
    const [selected, setSelected] = useState('main');
    
    function onSelected(name: string) {
      setSelected(name);
      props.sampleSelected(name);
    }
  
    return (
      <nav class="nav-column">
        <div class={"nav-1 nav-selectable" + (selected === 'main' ? " nav-current" : "")} onClick={() => onSelected('main')}>Editor</div>

        <div class="nav-1">Samples</div>

        {Object.keys(samples).filter(e => e !== 'main').map(name => (
          <div class={"nav-2 nav-selectable" + (selected === name ? " nav-current" : "")} onClick={() => onSelected(name)}>{name}</div>
        ))}

        <div class="nav-1">Katas</div>
        {katas.map(name => (<div class="nav-2 nav-selectable">{name}</div>))}
      </nav>);
}
