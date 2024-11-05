// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

const excludeTargets: string[] = [
  "ionq.qpu",
  "rigetti.qpu.aspen-m-2",
  "rigetti.qpu.ankaa-2",
];

const excludeProviders: string[] = [];

export function targetSupportQir(target: string) {
  // Note: Most of these should be dynamic at some point, with configuration coming
  // from the service, and able to be overridden by settings.
  return (
    target.startsWith("ionq") ||
    target.startsWith("quantinuum") ||
    target.startsWith("rigetti")
  );
}

export function shouldExcludeTarget(target: string) {
  return excludeTargets.includes(target);
}

export function shouldExcludeProvider(provider: string) {
  return excludeProviders.includes(provider);
}

export function supportsAdaptive(target: string) {
  return target.startsWith("quantinuum");
}
