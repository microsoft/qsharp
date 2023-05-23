// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget, VSDiagnostic } from "qsharp";
import { StateUpdater, useState } from "preact/hooks";
import { ResultsTab } from "./results.js";

function HirTab(props: { evtTarget: QscEventTarget; activeTab: string }) {
  const evtTarget = props.evtTarget;
  const hir = evtTarget.getHir();

  return props.activeTab === "hir-tab" ? (
    <pre class="hir-output">{hir}</pre>
  ) : null;
}

function TabNavItem(props: {
  id: string;
  title: string;
  activeTab: string;
  setActiveTab: StateUpdater<string>;
}) {
  const handleClick = () => {
    props.setActiveTab(props.id);
  };

  return (
    <div
      id={props.id}
      onClick={handleClick}
      class={props.activeTab === props.id ? "results-active-tab" : ""}
    >
      {props.title}
    </div>
  );
}

export function OutputTabs(props: {
  evtTarget: QscEventTarget;
  showPanel: boolean;
  onShotError?: (err?: VSDiagnostic) => void;
  kataMode?: boolean;
}) {
  const [activeTab, setActiveTab] = useState("results-tab");

  return (
    <div class="results-column">
      {props.showPanel ? (
        <div class="results-labels">
          <TabNavItem
            id="results-tab"
            title="RESULTS"
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
          <TabNavItem
            id="hir-tab"
            title="HIR"
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
          <TabNavItem
            id="logs-tab"
            title="LOGS"
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
        </div>
      ) : null}
      <ResultsTab {...props} activeTab={activeTab} />
      <HirTab {...props} activeTab={activeTab} />
    </div>
  );
}
