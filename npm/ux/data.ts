// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export type ReData = {
  status: string;
  jobParams: any;
  physicalCounts: any;
  physicalCountsFormatted: any;
  logicalQubit: any;
  tfactory: any;
  errorBudget: any;
  logicalCounts: any;
  new: boolean;
  frontierEntries: FrontierEntry[];
};

export type FrontierEntry = {
  logicalQubit: any;
  tfactory: any;
  errorBudget: any;
  physicalCounts: any;
  physicalCountsFormatted: any;
};

export function CreateReData(
  input: ReData,
  frontierEntryIndex: number | null = null,
): ReData {
  if (
    input.frontierEntries == null ||
    input.frontierEntries.length === 0 ||
    frontierEntryIndex == null
  ) {
    return input;
  } else {
    if (
      frontierEntryIndex < 0 ||
      frontierEntryIndex >= input.frontierEntries.length
    ) {
      frontierEntryIndex = 0;
    }

    const entry = input.frontierEntries[frontierEntryIndex];
    return {
      status: input.status,
      jobParams: input.jobParams,
      physicalCounts: entry.physicalCounts,
      physicalCountsFormatted: entry.physicalCountsFormatted,
      logicalQubit: entry.logicalQubit,
      tfactory: entry.tfactory,
      errorBudget: entry.errorBudget,
      logicalCounts: input.logicalCounts,
      frontierEntries: input.frontierEntries,
      new: input.new,
    };
  }
}
