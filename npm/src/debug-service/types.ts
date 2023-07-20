// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export class Span {
  constructor(public lo: number, public hi: number) {}
}

export class BreakpointSpan {
  constructor(public id: number, public lo: number, public hi: number) {}
}

export class StackFrame {
  constructor(
    public name: string,
    public path: string,
    public lo: number,
    public hi: number
  ) {}
}
