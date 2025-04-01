import { useState } from "preact/hooks";
import { Nav } from "../nav";

export type SideMenuProps = {
  selected: string;
  navSelected: (name: string) => void;
  katas: string[];
  samples: string[];
  namespaces: string[];
};

export function SideMenu({
  selected,
  navSelected,
  katas,
  samples,
  namespaces,
}: SideMenuProps) {
  const [isOpen, setIsOpen] = useState(true);

  const toggleMenu = () => setIsOpen(!isOpen);

  return (
    <>
      <header class="page-header">
        <div class="icon-row">
          <svg
            onClick={toggleMenu}
            width="32px"
            height="32px"
            viewBox="0 0 24 16"
            fill="none"
          >
            <title>Menu</title>
            <path
              d="M4 6H20M4 12H20M4 18H20"
              stroke="currentColor"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
        <div class="title-header">Q# Playground</div>
      </header>

      <div
        class={`qs-play-body ${isOpen ? "nav-column-open" : "nav-column-closed"}`}
      >
        <Nav
          selected={selected}
          navSelected={navSelected}
          katas={katas}
          samples={samples}
          namespaces={namespaces}
          sidebarOpen={isOpen}
        />
      </div>
    </>
  );
}
