export function Nav(props: {
  selected: string;
  navSelected: (name: string) => void;
  katas: string[];
  samples: string[];
  namespaces: string[];
  sidebarOpen: boolean;
}) {
  function onSelected(name: string) {
    props.navSelected(name);
  }

  return (
    <>
      <nav className={`nav-column ${props.sidebarOpen ? "open" : "closed"}`}>
          <>
            <div className="nav-1">Samples</div>
            {props.samples.map((name) => (
              <div
                className={`nav-2 nav-selectable ${
                  props.selected === "sample-" + name ? "nav-current" : ""
                }`}
                onClick={() => onSelected("sample-" + name)}
              >
                {name}
              </div>
            ))}

            <div className="nav-1">Tutorials</div>
            {props.katas.map((name) => (
              <div
                className={`nav-2 nav-selectable ${
                  props.selected === name ? "nav-current" : ""
                }`}
                onClick={() => onSelected(name)}
              >
                {name}
              </div>
            ))}

            <div className="nav-1">Documentation</div>
            {props.namespaces.map((name) => (
              <div
                className={`nav-2 nav-selectable ${
                  props.selected === name ? "nav-current" : ""
                }`}
                onClick={() => onSelected(name)}
              >
                {name}
              </div>
            ))}
          </>
      </nav>
    </>
  );
}
