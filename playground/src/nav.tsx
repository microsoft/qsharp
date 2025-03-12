// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function Nav(props: {
  selected: string;
  navSelected: (name: string) => void;
  katas: string[];
  samples: string[];
  namespaces: string[];
  theme?: "light" | "dark";
  onThemeChange?: (theme: "light" | "dark") => void;
}) {
  function onSelected(name: string) {
    props.navSelected(name);
  }

  function toggleTheme() {
    if (props.onThemeChange) {
      props.onThemeChange(props.theme === "light" ? "dark" : "light");
    }
  }

  return (
    <nav class="nav-column">
      <div class="nav-header">
        <div class="nav-1">Samples</div>
        {props.onThemeChange && (
          <div class="theme-toggle" onClick={toggleTheme}>
            <svg
              width="20px"
              height="20px"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              {props.theme === "light" ? (
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
              ) : (
                <>
                  <circle cx="12" cy="12" r="5"></circle>
                  <line x1="12" y1="1" x2="12" y2="3"></line>
                  <line x1="12" y1="21" x2="12" y2="23"></line>
                  <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                  <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                  <line x1="1" y1="12" x2="3" y2="12"></line>
                  <line x1="21" y1="12" x2="23" y2="12"></line>
                  <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                  <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                </>
              )}
            </svg>
          </div>
        )}
      </div>
      {props.samples.map((name) => (
        <div
          class={
            "nav-2 nav-selectable" +
            (props.selected === "sample-" + name ? " nav-current" : "")
          }
          onClick={() => onSelected("sample-" + name)}
        >
          {name}
        </div>
      ))}

      <div class="nav-1">Tutorials</div>
      {props.katas.map((name) => (
        <div
          class={
            "nav-2 nav-selectable" +
            (props.selected === name ? " nav-current" : "")
          }
          onClick={() => onSelected(name)}
        >
          {name}
        </div>
      ))}

      <div class="nav-1">Documentation</div>
      {props.namespaces.map((name) => (
        <div
          class={
            "nav-2 nav-selectable" +
            (props.selected === name ? " nav-current" : "")
          }
          onClick={() => onSelected(name)}
        >
          {name}
        </div>
      ))}
    </nav>
  );
}
