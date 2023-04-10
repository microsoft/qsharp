// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {writeFileSync}  from "node:fs";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {inspect} from "node:util";

console.log("CESARZC: BUILD");

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasMetadata = join(thisDir, "dist", "katas-metadata.js");

console.log(katasMetadata);
var obj = {a: 1, b: 2};
writeFileSync(katasMetadata, 'var obj = ' + inspect(obj) , 'utf-8');