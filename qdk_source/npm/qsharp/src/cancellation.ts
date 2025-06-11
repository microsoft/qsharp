// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export interface CancellationToken {
  // A flag signalling if cancellation has been requested.
  readonly isCancellationRequested: boolean;

  //  An event which fires when cancellation is requested.
  readonly onCancellationRequested: (listener: (e: any) => any) => void;
}

class InternalToken implements CancellationToken {
  private eventTarget: EventTarget;
  isCancellationRequested = false;

  constructor() {
    this.eventTarget = new EventTarget();
  }

  onCancellationRequested(listener: (e: any) => any) {
    this.eventTarget.addEventListener("cancelled", listener);
  }

  cancel() {
    if (this.isCancellationRequested) return; // Only fires once

    this.isCancellationRequested = true;
    this.eventTarget.dispatchEvent(new Event("cancelled"));
  }
}

export class CancellationTokenSource {
  private _token: InternalToken;

  constructor(parent?: CancellationToken) {
    // There are some optimizations you can do here to lazily allocate, but keeping it simple for now.
    this._token = new InternalToken();
    if (parent) parent.onCancellationRequested(() => this.cancel());
  }

  get token(): CancellationToken {
    return this._token;
  }

  cancel(): void {
    this._token.cancel();
  }
}
