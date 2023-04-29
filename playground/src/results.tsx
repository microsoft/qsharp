// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { Dump, QscEventTarget, ShotResult } from "qsharp";
import { useEffect, useState } from "preact/hooks"

const dumpExample: Dump = {"|0000⟩":[0.4338641278491088,-0.0655721495701471],"|0001⟩":[0.4338641278491088,0.0655721495701471],"|0010⟩":[0.4338641278491088,-0.0655721495701471],"|0011⟩":[0.4338641278491088,0.0655721495701471],"|1100⟩":[-0.03582222857458079,-0.23702105329787282],"|1101⟩":[0.03582222857458079,-0.23702105329787282],"|1110⟩":[-0.03582222857458079,-0.23702105329787282],"|1111⟩":[0.03582222857458079,-0.23702105329787282]};

function probability(real: number, imag: number) {
    return (real * real + imag * imag);
}

function formatComplex(real: number, imag: number) {
    // toLocaleString() correctly identifies -0 in JavaScript
    // String interpolation drops minus sign from -0
    // &#x2212; is the unicode minus sign, &#x1D456; is the mathematical i
    const realPart = `${real.toLocaleString()[0] === "-" ? "−" : ""}${Math.abs(real).toFixed(4)}`;
    const imagPart = `${imag.toLocaleString()[0] === "-" ? "−" : "+"}${Math.abs(imag).toFixed(4)}𝑖`;
    return `${realPart}${imagPart}`;
}

export function StateTable(props: {dump: Dump}) {
    return (
<table class="state-table">
  <thead>
    <tr>
      <th>Basis State<br/>(|𝜓ₙ…𝜓₁⟩)</th>
      <th>Amplitude</th>
      <th>Measurement Probability</th>
      <th colSpan={2}>Phase</th>
    </tr>
  </thead>
  <tbody>
{ Object.keys(props.dump).map(basis => {
    const [real, imag] = props.dump[basis];
    const complex = formatComplex(real, imag)
    const probabilityPercent = probability(real, imag) * 100;
    const phase = Math.atan2(imag, real);
    const phaseStyle = `transform: rotate(${phase.toFixed(4)}rad)`;
    return (
    <tr>
      <td style="text-align: center">{basis}</td>
      <td style="text-align: right">{complex}</td>
      <td style="display: flex; justify-content: space-between; padding: 8px 20px;">
        <progress style="width: 40%" max="100" value={probabilityPercent}></progress>
        <span>{probabilityPercent.toFixed(4)}%</span>
      </td>
      <td style={phaseStyle}>↑</td>
      <td style="text-align:right">{phase.toFixed(4)}</td>
    </tr>);
})}
  </tbody>
</table>);
}

type ResultsState = {
    shotCount: number;                  // How many shots have started
    resultCount: number;                // How many have completed (may be one less than above)
    currIndex: number;                  // Which is currently being displayed
    currResult: ShotResult | undefined; // The shot data to display
    buckets: Map<string, number>;       // Histogram buckets
    currArray: ShotResult[]             // Used to detect a new run
};

function newRunState() {
    return {
        shotCount: 0,
        resultCount: 0,
        currIndex: 0,
        currResult: undefined,
        buckets: new Map(),
        currArray: []
    };
}

function resultIsSame(a: ShotResult, b: ShotResult): boolean {
    // If the length changed, any entries are different objects, or the final result has changed.
    if (a.success !== b.success ||
        a.result !== b.result ||
        a.events.length !== b.events.length) return false;
    
    for(let i = 0; i < a.events.length; ++i) {
        if (a.events[i] !== b.events[i]) return false;
    }

    return true;
}

export function Results(props: {evtTarget: QscEventTarget}) {
    const [resultState, setResultState] = useState<ResultsState>(newRunState());

    // This is more complex than ideal for performance reasons. During a run, results may be getting
    // updated thousands of times a second, but there is no point trying to render at more than 60fps.
    // Therefore this subscribes to an event that happens once a frame if changes to results occur.
    // As the results are mutated array, they don't make good props or state, so need to manually
    // check for changes that would impact rendering and update state by creating new objects.
    const evtTarget = props.evtTarget;
    useEffect( () => {
        const resultUpdateHandler = () => {
            const results = evtTarget.getResults();

            // If it's a new run, the entire results array will be a new object
            const isNewResults = results !== resultState.currArray;

            // If the results object has change then reset the current index
            let newIndex = isNewResults ? 0 : resultState.currIndex;

            const currentResult = resultState.currResult;
            const updatedResult = newIndex < results.length ?
                    results[newIndex] : undefined;

            const replaceResult = isNewResults ||
                    // One is defined but the other isn't
                    (!currentResult !== !updatedResult) ||
                    // Or they both exist but are different
                    (currentResult && updatedResult && !resultIsSame(currentResult, updatedResult));
            
            // Keep the old object if no need to replace it, else construct a new one
            const newResult = !replaceResult ? currentResult :
                !updatedResult ? undefined : {
                    success: updatedResult.success,
                    result: updatedResult.result,
                    events: [...updatedResult.events]
                };
            
            // Update the histogram if new results have come in.
            // For now, just completely recreate the bucket map
            const resultCount = evtTarget.resultCount();
            let buckets = resultState.buckets;
            // If there are entirely new results, or if new results have been added, recalculate.
            if (isNewResults || resultState.resultCount !== resultCount) {
                buckets = new Map();
                for(let i = 0; i < resultCount; ++i) {
                    const key = results[i].result;
                    const strKey = typeof key === 'string' ? key : "ERROR";
                    const newValue = (buckets.get(strKey) || 0) + 1;
                    buckets.set(strKey, newValue);
                }
            }

            // If anything needs updating, construct the new state object and store
            if (replaceResult ||
                    resultState.shotCount !== results.length || 
                    resultState.resultCount !== resultCount ||
                    resultState.currIndex !== newIndex) {
                setResultState({
                    shotCount: results.length,
                    resultCount: resultCount,
                    currIndex: newIndex,
                    currResult: newResult,
                    buckets,
                    currArray: results
                });
            }
        };
    
        evtTarget.addEventListener('uiResultsRefresh', resultUpdateHandler);

        // Remove the event listener when this component is destroyed
        return () => evtTarget.removeEventListener('uiResultsRefresh', resultUpdateHandler)
    }, [evtTarget])

    return (
<div class="results-column">
  <div class="results-labels">
    <div class="results-active-tab">RESULTS</div>
    <div>AST</div>
    <div>LOGS</div>
  </div>
  <div id="histogram">
    {[...resultState.buckets].map( ([key,val])=>(
        <div>Bucket: {key}, Count: {val}</div>
    ))}
  </div>
  <div class="output-header">
    <div>Shot {resultState.currIndex} of {resultState.shotCount}</div>
    <div class="prev-next">Prev | Next</div>
  </div>
    <div>
        {resultState.currResult?.events.map(evt => {
            return evt.type === "Message" ? 
                (<div>Message: {evt.message}</div>) : 
                (<StateTable dump={evt.state}></StateTable>)
        })}
    </div>
  
</div>);
}
