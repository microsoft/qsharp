// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Note: Most of these should be dynamic at some point, with configuration coming
// from the service, and able to be overridden by settings.

const targetsThatSupportQir = [
  "quantinuum.sim.h1-1sc",
  "quantinuum.sim.h1-1e",
  "quantinuum.qpu.h1-1",
  "quantinuum.sim.h1-2sc",
  "quantinuum.sim.h1-2e",
  "quantinuum.qpu.h1-2",
  "quantinuum.sim.h2-1sc",
  "quantinuum.sim.h2-1e",
  "quantinuum.qpu.h2-1",
  "rigetti.sim.qvm",
  "rigetti.qpu.aspen-m-3",
  "ionq.qpu-preview",
  "ionq.qpu.aria-1-preview",
  "ionq.qpu.aria-2-preview",
  "ionq.simulator-preview",
];

const excludeTargets: string[] = ["rigetti.qpu.aspen-m-2", "rigetti.echo"];

const excludeProviders: string[] = ["microsoft-qc", "Microsoft.Test"];

export function targetSupportQir(target: string) {
  return targetsThatSupportQir.includes(target);
}

export function shouldExcludeTarget(target: string) {
  return excludeTargets.includes(target);
}

export function shouldExcludeProvider(provider: string) {
  return excludeProviders.includes(provider);
}
