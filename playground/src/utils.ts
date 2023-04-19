// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Utility functions to convert source code to and from base64.
//
// The btoa function expects a string of bytes (i.e. in the range x00 - xFF), however
// as the code may contain UTF-16 code units outside this range, it needs to be converted
// first. If you just encodeUriComponent the whole string, it will blow up considerably as
// all spaces and newlines (common chars in code) become %20 and %0A (so 4 spaces and a newline
// and up being 15 chars) whereas base64 encoding those 5 chars in ASCII is just 'ICAgIAo' (i.e.
// 7 bytes, or less then half the size).
//
// The below functions simply convert the source to utf-8 bytes first (and as most source code
// is ASCII this is usually a one-to-one mapping), and then base64 encodes/decodes the utf-8
// bytes. In testing this results in an encoding about half the size of other methods.

export function codeToBase64(code: string) : string {
  // Convert to utf=8
  const myencoder = new TextEncoder();
  const buff = myencoder.encode(code);

  // Create a string of the utf-8 code units (so each will be <= 0xFF)
  let binStr = "";
  for(const unit of buff) {
    binStr += String.fromCharCode(unit);
  }

  // Convert that to base64
  const base64String = window.btoa(binStr);
  return base64String;
}

export function base64ToCode(b64: string) : string {
  // Get the binary string of utf-8 code units
  const binStr = window.atob(b64);

  // Create the Uint8Array from it
  const byteArray = new Uint8Array(binStr.length);
  for (let i = 0; i < binStr.length; ++i) byteArray[i] = binStr.charCodeAt(i);

  // Decode the utf-8 bytes into a JavaScript string
  const decoder = new TextDecoder();
  const code = decoder.decode(byteArray);
  return code;
}
