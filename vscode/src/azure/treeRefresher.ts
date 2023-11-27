// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { log } from "qsharp-lang";
import { WorkspaceConnection, WorkspaceTreeProvider } from "./treeView";

// To ensure that for a given workspace, we don't have multiple refreshes running
const runningTimeouts = new Map<string, any>();

function stopTimeout(workspaceId: string) {
  if (runningTimeouts.has(workspaceId)) {
    clearTimeout(runningTimeouts.get(workspaceId));
    runningTimeouts.delete(workspaceId);
  }
}

/**
 * @param minMs - Initial refresh delay
 * @param maxMs - Maximum refresh delay
 * @param stepFactor - Refresh delay multiplier
 * @param limit - Maximum total time to keep refreshing
 * @returns A generator that yields the next refresh delay
 */
function* refreshRangeIterator(
  minMs: number,
  maxMs: number,
  stepFactor: number,
  limit: number,
) {
  let totalMs = 0;
  do {
    yield minMs;
    totalMs += minMs;
    minMs = Math.min(minMs * stepFactor, maxMs);
  } while (totalMs < limit);
}

// Iterate with the delays until the predicate is true for the workspace
async function iterateUntilTrue(
  iter: Generator<number>,
  id: string,
  predicate: () => Promise<boolean>,
  onDone?: () => void,
) {
  const nextDelay = iter.next();
  if (nextDelay.value && !(await predicate())) {
    // Schedule the next iteration
    log.debug("Scheduling next workspace refresh in ", nextDelay.value, "ms");
    const timeoutId = setTimeout(
      () => iterateUntilTrue(iter, id, predicate, onDone),
      nextDelay.value,
    );
    runningTimeouts.set(id, timeoutId);
  } else {
    // Either reached the time limit or the predicate is true and we're done
    if (!nextDelay.value) {
      log.debug("Reached time limit for workspace refresh");
    } else {
      log.debug("Predicate is true for workspace refresh. Done refreshing");
    }
    runningTimeouts.delete(id);
    if (onDone) onDone();
  }
}

export function startRefreshCycle(
  treeProvider: WorkspaceTreeProvider,
  workspace: WorkspaceConnection,
  newJobId?: string,
  onDone?: () => void,
) {
  log.debug(
    "Refreshing jobs list until they are all finished for workspace: ",
    workspace.id,
  );
  // Stop any other refreshes for this workspace
  stopTimeout(workspace.id);

  // Initial refresh at 5 seconds, backing off up to 5 minutes between requests (doubling the latency each time),
  // and only keep refreshing for up to 1 hour, or until all jobs report as completed
  const iter = refreshRangeIterator(5000, 5 * 60 * 1000, 2, 60 * 60 * 1000);
  const predicate = async () => {
    if (!treeProvider.hasWorkspace(workspace.id)) return true;

    try {
      await treeProvider.refreshWorkspace(workspace);
    } catch (e: any) {
      log.error("Error refreshing in workspace refresh cycle: ", e);
      // The above could throw due to transient network errors, etc. so just keep trying next time
      return false;
    }

    // If we're waiting for a new job to appear, check for that
    if (newJobId) {
      if (!treeProvider.workspaceHasJob(workspace.id, newJobId)) {
        log.debug("Still waiting for new job to appear: ", newJobId);
        return false;
      }
    }

    // Else just check if there are any pending jobs to keep checking
    if (treeProvider.hasJobsPending(workspace.id)) {
      log.debug("Still waiting for jobs to complete");
      return false;
    } else {
      log.debug("All jobs for the workspace showing as completed");
      return true;
    }
  };

  iterateUntilTrue(iter, workspace.id, predicate, onDone);
}
