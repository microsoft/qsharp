// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// base64 functions to encode unicode
// Per samples on https://developer.mozilla.org/en-US/docs/Web/API/btoa

// convert a Unicode string to a string in which
// each 16-bit unit occupies only one byte
// e.g.
//   const myString = "☸☹☺☻☼☾☿";
//   const converted = toBinary(myString);   // "8&9&:&;&<&>&?&"
//   const encoded = window.btoa(converted); // "OCY5JjomOyY8Jj4mPyY="
export function toBinary(string: string) {
  const codeUnits = Uint16Array.from(
    { length: string.length },
    (element, index) => string.charCodeAt(index)
  );
  const charCodes = new Uint8Array(codeUnits.buffer);

  let result = "";
  charCodes.forEach((char) => {
    result += String.fromCharCode(char);
  });
  return result;
}

// Does the inverse of toBinary, e.g.
//   const decoded = window.atob(encoded); // "8&9&:&;&<&>&?&"
//   const original = fromBinary(decoded); // "☸☹☺☻☼☾☿"
export function fromBinary(binary: string) {
  const bytes = Uint8Array.from({ length: binary.length }, (element, index) =>
    binary.charCodeAt(index)
  );
  const charCodes = new Uint16Array(bytes.buffer);

  let result = "";
  charCodes.forEach((char) => {
    result += String.fromCharCode(char);
  });
  return result;
}
