// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export function Results(props: any) {
    return (
<div class="results-column">
  <div class="results-labels">
    <div class="results-active-tab">RESULTS</div>
    <div>AST</div>
    <div>LOGS</div>
  </div>
  <div id="histogram"></div>
  <div class="output-header">
    <div>Shot 21 of 100</div>
    <div class="prev-next">Prev | Next</div>
  </div>
</div>);
}
