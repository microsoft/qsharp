// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget, VSDiagnostic } from "qsharp";
import { ResultsTab } from "./results.js";
import { useState } from "preact/hooks";

export type ActiveTab = "results-tab" | "hir-tab" | "logs-tab";

function HirTab(props: { hir: string; activeTab: ActiveTab }) {
  return props.activeTab === "hir-tab" ? (
    <textarea readonly class="hir-output">
      {props.hir}
    </textarea>
  ) : null;
}

function TabNavItem(props: {
  id: string;
  title: string;
  activeTab: string;
  setActiveTab: (tab: ActiveTab) => void;
}) {
  const handleClick = () => {
    switch (props.id) {
      case "results-tab":
      case "hir-tab":
      case "logs-tab":
        props.setActiveTab(props.id as ActiveTab);
        break;
      default:
        props.setActiveTab("results-tab");
    }
  };

  return (
    <div
      id={props.id}
      onClick={handleClick}
      class={props.activeTab === props.id ? "active-tab" : ""}
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
  hir: string;
}) {
  const [activeTab, setActiveTab] = useState<ActiveTab>("results-tab");

  return (
    <div class="results-column">
      {props.showPanel ? (
        <div class="results-labels">
          <TabNavItem
            {...props}
            id="results-tab"
            title="RESULTS"
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
          <TabNavItem
            {...props}
            id="hir-tab"
            title="HIR"
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
          <TabNavItem
            {...props}
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
