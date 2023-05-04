// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function Nav(props: {
      selected: string, 
      navSelected: (name: string) => void,
      katas: string[], 
      samples: string[]}) {
    
    function onSelected(name: string) {
      props.navSelected(name);
    }
  
    return (
      <nav class="nav-column">
        <div class="nav-1">Samples</div>

        {props.samples.map(name => (
          <div class={"nav-2 nav-selectable" + (props.selected === name ? " nav-current" : "")}
               onClick={() => onSelected(name)}>{name}
          </div>
        ))}

        <div class="nav-1">Katas</div>
        {props.katas.map(name => (
          <div class={"nav-2 nav-selectable" + (props.selected === name ? " nav-current" : "")}
               onClick={() => onSelected(name)}>{name}
          </div>))}
      </nav>);
}
