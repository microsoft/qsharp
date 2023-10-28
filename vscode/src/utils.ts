// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Guid format such as "00000000-1111-2222-3333-444444444444"
export function getRandomGuid(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));

  // Per https://www.ietf.org/rfc/rfc4122.txt, for UUID v4 (random GUIDs):
  // - Octet 6 contains the version in top 4 bits (0b0100)
  // - Octet 8 contains the variant in the top 2 bits (0b10)
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  // Convert the 16 bytes into 32 hex digits
  const hex = bytes.reduce(
    (acc, byte) => acc + byte.toString(16).padStart(2, "0"),
    "",
  );

  return (
    hex.substring(0, 8) +
    "-" +
    hex.substring(8, 12) +
    "-" +
    hex.substring(12, 16) +
    "-" +
    hex.substring(16, 20) +
    "-" +
    hex.substring(20, 32)
  );
}
