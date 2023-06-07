// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { QscEventTarget, VSDiagnostic } from "qsharp";
import { ResultsTab } from "./results.js";
import { ActiveTab } from "./main.js";

const tabArray: Array<[ActiveTab, string]> = [
  ["results-tab", "RESULTS"],
  ["hir-tab", "HIR"],
  ["logs-tab", "LOGS"],
];

function HirTab(props: { hir: string; activeTab: ActiveTab }) {
  return props.activeTab === "hir-tab" ? (
    <textarea readonly class="hir-output">
      {props.hir}
    </textarea>
  ) : null;
}

export function OutputTabs(props: {
  evtTarget: QscEventTarget;
  showPanel: boolean;
  onShotError?: (err?: VSDiagnostic) => void;
  kataMode?: boolean;
  hir: string;
  activeTab: ActiveTab;
  setActiveTab: (tab: ActiveTab) => void;
}) {
  return (
    <div class="results-column">
      {props.showPanel ? (
        <div class="results-labels">
          {tabArray.map((elem) => (
            <div
              onClick={() => props.setActiveTab(elem[0])}
              class={props.activeTab === elem[0] ? "active-tab" : ""}
            >
              {elem[1]}
            </div>
          ))}
        </div>
      ) : null}
      <ResultsTab {...props} />
      <HirTab {...props} />
    </div>
  );
}
